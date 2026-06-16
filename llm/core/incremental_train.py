#!/usr/bin/env python3
"""Continue LoRA training from an existing adapter using approved gameplay data."""

from __future__ import annotations

import argparse
import json
import random
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from project_loader import get_project_dir, load_project, register_model, require_base_model  # noqa: E402

from train_lora import format_chat, load_jsonl, train  # noqa: E402
from train_utils import bitsandbytes_config, cuda_device_map, require_cuda, resolve_precision  # noqa: E402


def build_incremental_dataset(
    project: str,
    replay_ratio: float = 0.2,
    max_new: int = 500,
    seed: int = 42,
) -> Path:
    manifest = load_project(project)
    project_dir = get_project_dir(project)
    train_cfg = manifest.get("train", {})

    approved_path = project_dir / "datasets" / "gameplay" / "approved.jsonl"
    base_train_path = project_dir / "datasets" / "train.jsonl"
    if not approved_path.exists():
        raise FileNotFoundError(f"No approved gameplay data: {approved_path}")

    new_examples = load_jsonl(approved_path)[: max_new]
    replay: list[dict] = []
    if base_train_path.exists():
        all_train = load_jsonl(base_train_path)
        random.seed(seed)
        replay_count = int(len(new_examples) * replay_ratio / max(0.01, 1 - replay_ratio))
        replay_count = min(replay_count, len(all_train))
        replay = random.sample(all_train, replay_count) if replay_count else []

    combined = new_examples + replay
    random.shuffle(combined)

    out_path = project_dir / "datasets" / "incremental_train.jsonl"
    out_path.write_text(
        "\n".join(json.dumps(e, ensure_ascii=False) for e in combined) + "\n",
        encoding="utf-8",
    )
    print(f"Incremental dataset: {len(new_examples)} new + {len(replay)} replay = {len(combined)}")
    return out_path


def incremental_train(
    project: str,
    from_adapter: str = "v1",
    output_adapter: str = "v2-personal",
    dry_run: bool = False,
) -> Path:
    manifest = load_project(project)
    project_dir = get_project_dir(project)
    train_cfg = manifest.get("train", {})

    from_dir = project_dir / "adapters" / from_adapter
    if not from_dir.exists():
        raise FileNotFoundError(f"Source adapter not found: {from_dir}")

    inc_path = build_incremental_dataset(
        project,
        replay_ratio=train_cfg.get("replay_ratio", 0.2),
        max_new=train_cfg.get("max_incremental_examples", 500),
    )

    # Temporarily point train at incremental set
    train_path = project_dir / "datasets" / "train.jsonl"
    backup = project_dir / "datasets" / "train.jsonl.bak"
    if train_path.exists():
        train_path.rename(backup)
    inc_path.rename(train_path)

    try:
        if dry_run:
            print(f"Dry run: would train {output_adapter} from {from_adapter}")
            examples = load_jsonl(train_path)
            print(f"  Examples: {len(examples)}")
            return project_dir / "adapters" / output_adapter

        # Custom train that loads from existing adapter
        try:
            import torch
            from datasets import Dataset
            from peft import PeftModel
            from transformers import AutoModelForCausalLM, AutoTokenizer
            from trl import SFTConfig, SFTTrainer
        except ImportError as e:
            raise ImportError(f"Training deps missing: {e}") from e

        require_cuda()
        base_model = require_base_model(manifest, project)
        examples = load_jsonl(train_path)
        output_dir = project_dir / "adapters" / output_adapter
        output_dir.mkdir(parents=True, exist_ok=True)

        bnb_config = bitsandbytes_config()
        tokenizer = AutoTokenizer.from_pretrained(base_model, trust_remote_code=True)
        if tokenizer.pad_token is None:
            tokenizer.pad_token = tokenizer.eos_token

        base = AutoModelForCausalLM.from_pretrained(
            base_model,
            quantization_config=bnb_config,
            device_map=cuda_device_map(),
            trust_remote_code=True,
        )
        model = PeftModel.from_pretrained(base, str(from_dir), is_trainable=True)

        texts = [format_chat(ex, tokenizer) for ex in examples]
        dataset = Dataset.from_dict({"text": texts})

        precision = resolve_precision()
        sft_config = SFTConfig(
            output_dir=str(output_dir / "checkpoint"),
            num_train_epochs=1,
            per_device_train_batch_size=train_cfg.get("batch_size", 4),
            gradient_accumulation_steps=train_cfg.get("gradient_accumulation_steps", 4),
            learning_rate=train_cfg.get("learning_rate", 2e-4) * 0.5,
            max_length=train_cfg.get("max_seq_len", 2048),
            logging_steps=5,
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
        print(f"Incremental train: {from_adapter} -> {output_adapter} ({len(examples)} examples)")
        trainer.train()
        model.save_pretrained(str(output_dir))
        tokenizer.save_pretrained(str(output_dir))

        config = {
            "parent_adapter": from_adapter,
            "incremental_examples": len(examples),
            "trained_at": datetime.now(timezone.utc).isoformat(),
        }
        (output_dir / "train_config.json").write_text(json.dumps(config, indent=2), encoding="utf-8")

        tag = f"{manifest.get('ollama_tag', project)}-personal"
        register_model(
            {
                "ollama_tag": tag,
                "project": project,
                "version": output_adapter,
                "parent": manifest.get("ollama_tag", project),
                "base_model": base_model,
                "adapter": f"projects/{project}/adapters/{output_adapter}",
                "created_at": config["trained_at"][:10],
                "eval_score": None,
            },
            set_active=True,
        )
        print(f"Saved adapter to {output_dir}")
        return output_dir

    finally:
        if backup.exists():
            if train_path.exists():
                train_path.unlink()
            backup.rename(train_path)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--project", required=True)
    parser.add_argument("--from-adapter", default="v1")
    parser.add_argument("--output", default="v2-personal")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    try:
        incremental_train(args.project, args.from_adapter, args.output, args.dry_run)
        return 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
