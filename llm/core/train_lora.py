#!/usr/bin/env python3
"""LoRA fine-tune a base model for a project."""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from lora_targets import resolve_lora_targets  # noqa: E402
from project_loader import get_project_dir, load_project, register_model, require_base_model  # noqa: E402
from train_utils import bitsandbytes_config, cuda_device_map, require_cuda, resolve_precision  # noqa: E402


def load_jsonl(path: Path) -> list[dict]:
    rows = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            rows.append(json.loads(line))
    return rows


def format_chat(example: dict, tokenizer=None) -> str:
    """Format messages into a training string."""
    messages = example.get("messages", [])
    if tokenizer is not None and hasattr(tokenizer, "apply_chat_template"):
        try:
            return tokenizer.apply_chat_template(
                messages, tokenize=False, add_generation_prompt=False
            )
        except Exception:
            pass
    parts = []
    for msg in messages:
        role = msg["role"]
        content = msg["content"]
        if role == "user":
            parts.append(f"<start_of_turn>user\n{content}<end_of_turn>")
        else:
            parts.append(f"<start_of_turn>model\n{content}<end_of_turn>")
    return "\n".join(parts) + "\n"


def train(
    project: str,
    adapter_version: str = "v1",
    dry_run: bool = False,
    max_samples: int | None = None,
    epochs: int | None = None,
) -> Path:
    manifest = load_project(project)
    project_dir = get_project_dir(project)
    train_cfg = manifest.get("train", {})

    train_path = project_dir / "datasets" / "train.jsonl"
    if not train_path.exists():
        raise FileNotFoundError(f"Missing {train_path}. Run build_dataset.py first.")

    adapter_dir = project_dir / "adapters" / adapter_version
    adapter_dir.mkdir(parents=True, exist_ok=True)

    base_model = require_base_model(manifest, project)
    fallback = manifest.get("fallback_base_model")
    examples = load_jsonl(train_path)
    if max_samples and max_samples < len(examples):
        examples = examples[:max_samples]
        print(f"Using {len(examples)} samples (--max-samples {max_samples})")

    config = {
        "project": project,
        "base_model": base_model,
        "adapter_version": adapter_version,
        "train_examples": len(examples),
        "lora_rank": train_cfg.get("lora_rank", 16),
        "lora_alpha": train_cfg.get("lora_alpha", 32),
        "epochs": epochs if epochs is not None else train_cfg.get("epochs", 3),
        "max_seq_len": train_cfg.get("max_seq_len", 2048),
        "learning_rate": train_cfg.get("learning_rate", 2e-4),
        "batch_size": train_cfg.get("batch_size", 4),
        "gradient_accumulation_steps": train_cfg.get("gradient_accumulation_steps", 4),
        "trained_at": datetime.now(timezone.utc).isoformat(),
    }

    if dry_run:
        print(json.dumps(config, indent=2))
        print(f"Dry run - would save adapter to {adapter_dir}")
        return adapter_dir

    try:
        from datasets import Dataset
        from peft import LoraConfig, get_peft_model, prepare_model_for_kbit_training
        from transformers import AutoModelForCausalLM, AutoTokenizer
        from trl import SFTConfig, SFTTrainer
    except ImportError as e:
        print(f"Training deps missing: {e}", file=sys.stderr)
        print("Install: pip install -r llm/core/requirements.txt", file=sys.stderr)
        raise

    require_cuda()
    print(f"Loading base model: {base_model}")
    bnb_config = bitsandbytes_config()
    device_map = cuda_device_map()

    def load_model_and_tokenizer(model_id: str):
        tok = AutoTokenizer.from_pretrained(model_id, trust_remote_code=True)
        if tok.pad_token is None:
            tok.pad_token = tok.eos_token
        mdl = AutoModelForCausalLM.from_pretrained(
            model_id,
            quantization_config=bnb_config,
            device_map=device_map,
            trust_remote_code=True,
        )
        return mdl, tok

    try:
        model, tokenizer = load_model_and_tokenizer(base_model)
    except Exception as e:
        if not fallback:
            raise
        print(f"Failed to load {base_model}: {e}")
        print(f"Falling back to {fallback}")
        base_model = fallback
        config["base_model"] = base_model
        model, tokenizer = load_model_and_tokenizer(base_model)

    model = prepare_model_for_kbit_training(model)

    target_modules = resolve_lora_targets(model, base_model)
    lora_config = LoraConfig(
        r=train_cfg.get("lora_rank", 16),
        lora_alpha=train_cfg.get("lora_alpha", 32),
        target_modules=target_modules,
        lora_dropout=0.05,
        bias="none",
        task_type="CAUSAL_LM",
    )
    try:
        model = get_peft_model(model, lora_config)
    except Exception as e:
        if not fallback or "Gemma4ClippableLinear" not in str(e):
            raise
        print(f"Gemma 4 LoRA failed ({e})")
        print(f"Falling back to {fallback}")
        base_model = fallback
        config["base_model"] = base_model
        model, tokenizer = load_model_and_tokenizer(base_model)
        model = prepare_model_for_kbit_training(model)
        lora_config = LoraConfig(
            r=train_cfg.get("lora_rank", 16),
            lora_alpha=train_cfg.get("lora_alpha", 32),
            target_modules=resolve_lora_targets(model, base_model),
            lora_dropout=0.05,
            bias="none",
            task_type="CAUSAL_LM",
        )
        model = get_peft_model(model, lora_config)

    texts = [format_chat(ex, tokenizer) for ex in examples]
    dataset = Dataset.from_dict({"text": texts})

    output_dir = adapter_dir / "checkpoint"
    precision = resolve_precision()
    sft_config = SFTConfig(
        output_dir=str(output_dir),
        num_train_epochs=config["epochs"],
        per_device_train_batch_size=train_cfg.get("batch_size", 4),
        gradient_accumulation_steps=train_cfg.get("gradient_accumulation_steps", 4),
        learning_rate=train_cfg.get("learning_rate", 2e-4),
        max_length=train_cfg.get("max_seq_len", 2048),
        logging_steps=10,
        save_strategy="epoch",
        report_to="none",
        dataset_text_field="text",
        **precision,
    )

    trainer = SFTTrainer(
        model=model,
        args=sft_config,
        train_dataset=dataset,
        processing_class=tokenizer,
    )

    print(f"Training on {len(examples)} examples...")
    trainer.train()
    model.save_pretrained(str(adapter_dir))
    tokenizer.save_pretrained(str(adapter_dir))
    (adapter_dir / "train_config.json").write_text(json.dumps(config, indent=2), encoding="utf-8")

    register_model(
        {
            "ollama_tag": manifest.get("ollama_tag", project),
            "project": project,
            "version": adapter_version,
            "base_model": base_model,
            "adapter": f"projects/{project}/adapters/{adapter_version}",
            "knowledge_manifest": f"projects/{project}/knowledge/_merged/manifest.json",
            "created_at": config["trained_at"][:10],
            "train_examples": len(examples),
            "eval_score": None,
        },
        set_active=True,
    )

    print(f"Adapter saved to {adapter_dir}")
    return adapter_dir


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--project", required=True)
    parser.add_argument("--adapter", default="v1", help="Adapter version directory name")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--max-samples", type=int, help="Limit training examples for quick runs")
    parser.add_argument("--epochs", type=int, help="Override epoch count")
    args = parser.parse_args()
    try:
        train(
            args.project,
            args.adapter,
            dry_run=args.dry_run,
            max_samples=args.max_samples,
            epochs=args.epochs,
        )
        return 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
