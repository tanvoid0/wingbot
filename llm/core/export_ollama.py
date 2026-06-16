#!/usr/bin/env python3
"""Export LoRA adapter to GGUF and create Ollama model."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from project_loader import (  # noqa: E402
    get_ollama_base_model,
    get_project_dir,
    load_project,
    register_model,
    require_base_model,
)


def run(cmd: list[str], cwd: Path | None = None) -> subprocess.CompletedProcess:
    print(f"$ {' '.join(cmd)}")
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)


def find_llama_convert() -> Path | None:
    for candidate in (
        shutil.which("convert_hf_to_gguf.py"),
        Path(r"C:\tools\llama.cpp\convert_hf_to_gguf.py"),
    ):
        if candidate and Path(candidate).exists():
            return Path(candidate)
    return None


def find_llama_quantize() -> Path | None:
    for candidate in (
        shutil.which("llama-quantize"),
        Path(r"C:\tools\llama.cpp\build\bin\Release\llama-quantize.exe"),
        Path(r"C:\tools\llama.cpp\build\bin\llama-quantize.exe"),
        Path(r"C:\tools\llama-bin\llama-quantize.exe"),
    ):
        if candidate and Path(candidate).exists():
            return Path(candidate)
    return None


def quant_type_for_llama_quantize(quant: str) -> str:
    """Map our quant names to llama-quantize type (e.g. q4_k_m -> Q4_K_M)."""
    return quant.upper().replace("-", "_")


def convert_merged_to_gguf(merged_dir: Path, gguf_path: Path, quant: str) -> tuple[bool, str]:
    """HF merged dir -> GGUF. New llama.cpp: f16 convert, then llama-quantize for K-quants."""
    llama_convert = find_llama_convert()
    if not llama_convert or not merged_dir.exists():
        return False, "llama.cpp convert_hf_to_gguf.py not found or merged dir missing"

    # Direct convert types supported by convert_hf_to_gguf.py
    direct_types = {"f32", "f16", "bf16", "q8_0", "tq1_0", "tq2_0", "auto"}
    if quant in direct_types:
        result = run(
            ["python", str(llama_convert), str(merged_dir), "--outfile", str(gguf_path), "--outtype", quant]
        )
        if result.returncode == 0:
            return True, f"GGUF written to {gguf_path}"
        return False, f"GGUF conversion failed: {result.stderr or result.stdout}"

    # K-quants (q4_k_m, q5_k_m, …): convert f16 first, then llama-quantize
    f16_path = gguf_path.with_name(gguf_path.stem.replace(f"-{quant}", "") + "-f16.gguf")
    if f16_path == gguf_path:
        f16_path = gguf_path.with_name(gguf_path.stem + "-f16.gguf")

    result = run(
        ["python", str(llama_convert), str(merged_dir), "--outfile", str(f16_path), "--outtype", "f16"]
    )
    if result.returncode != 0:
        return False, f"F16 GGUF conversion failed: {result.stderr or result.stdout}"

    llama_quantize = find_llama_quantize()
    if not llama_quantize:
        return False, (
            f"F16 GGUF at {f16_path}, but llama-quantize not found. "
            f"Build llama.cpp, then run:\n"
            f"  llama-quantize {f16_path} {gguf_path} {quant_type_for_llama_quantize(quant)}"
        )

    qtype = quant_type_for_llama_quantize(quant)
    result = run([str(llama_quantize), str(f16_path), str(gguf_path), qtype])
    if result.returncode == 0:
        return True, f"GGUF written to {gguf_path} (quantized {qtype})"
    return False, f"Quantization failed: {result.stderr or result.stdout}"


def merge_and_export_gguf(
    project: str,
    adapter_version: str = "v1",
    quant: str = "q4_k_m",
    skip_merge: bool = False,
) -> Path:
    manifest = load_project(project)
    project_dir = get_project_dir(project)
    adapter_dir = project_dir / "adapters" / adapter_version
    export_dir = project_dir / "export"
    export_dir.mkdir(parents=True, exist_ok=True)

    base_model = require_base_model(manifest, project)
    ollama_base = get_ollama_base_model()
    ollama_tag = manifest.get("ollama_tag", project)
    gguf_name = f"{ollama_tag}-{quant}.gguf"
    gguf_path = export_dir / gguf_name
    merged_dir = export_dir / "merged"

    instructions_path = export_dir / "EXPORT_INSTRUCTIONS.md"

    if not adapter_dir.exists():
        raise FileNotFoundError(f"Adapter not found: {adapter_dir}. Run train_lora.py first.")

    steps = []

    if not skip_merge:
        # Try merging with peft + transformers
        try:
            import torch
            from peft import PeftModel
            from transformers import AutoModelForCausalLM, AutoTokenizer

            print("Merging LoRA adapter into base weights...")
            tokenizer = AutoTokenizer.from_pretrained(base_model, trust_remote_code=True)
            base = AutoModelForCausalLM.from_pretrained(
                base_model,
                torch_dtype=torch.bfloat16,
                device_map="cpu",
                trust_remote_code=True,
            )
            model = PeftModel.from_pretrained(base, str(adapter_dir))
            model = model.merge_and_unload()
            merged_dir.mkdir(parents=True, exist_ok=True)
            model.save_pretrained(str(merged_dir))
            tokenizer.save_pretrained(str(merged_dir))
            steps.append(f"Merged weights saved to {merged_dir}")
        except ImportError:
            steps.append("Install torch/peft/transformers to auto-merge. See EXPORT_INSTRUCTIONS.md.")

    # GGUF conversion requires llama.cpp — two-step for K-quants (f16 -> llama-quantize)
    if merged_dir.exists():
        ok, msg = convert_merged_to_gguf(merged_dir, gguf_path, quant)
        steps.append(msg)
        if not ok and not gguf_path.exists():
            gguf_path = export_dir / f"{ollama_tag}-q4.gguf"
    else:
        steps.append(
            "GGUF conversion not run (merged weights missing). "
            "Run without --skip-merge first."
        )
        gguf_path = export_dir / f"{ollama_tag}-q4.gguf"

    # Write Modelfile with correct FROM
    modelfile_src = export_dir / "Modelfile"
    system_path = project_dir / manifest.get("system_prompt", "export/system.txt")
    system_text = system_path.read_text(encoding="utf-8").strip() if system_path.exists() else ""

    if gguf_path.exists():
        from_line = f"FROM ./{gguf_path.name}"
    else:
        from_line = f"FROM {ollama_base}"
        steps.append(f"Using base {ollama_base} in Modelfile until custom GGUF is available.")

    modelfile_content = f"""{from_line}

PARAMETER temperature 0.4
PARAMETER top_p 0.9
PARAMETER num_ctx 4096

SYSTEM \"\"\"{system_text}\"\"\"
"""
    modelfile_src.write_text(modelfile_content, encoding="utf-8")

    # Try ollama create
    if shutil.which("ollama"):
        result = run(["ollama", "create", ollama_tag, "-f", str(modelfile_src)])
        if result.returncode == 0:
            steps.append(f"Ollama model '{ollama_tag}' created.")
        else:
            steps.append(f"ollama create failed: {result.stderr or result.stdout}")
    else:
        steps.append("Ollama not in PATH — install from https://ollama.com")

    instructions = f"""# Export instructions for {project}

Generated: {datetime.now(timezone.utc).isoformat()}

## Steps completed

{chr(10).join('- ' + s for s in steps)}

## Manual GGUF export (if auto failed)

1. Merge adapter:
   ```bash
   python -c "from peft import PeftModel; ..."
   ```
2. Convert to F16 GGUF, then quantize (llama.cpp no longer accepts q4_k_m on convert):
   ```powershell
   python C:\tools\llama.cpp\convert_hf_to_gguf.py {merged_dir} --outfile {export_dir / (ollama_tag + "-f16.gguf")} --outtype f16
   C:\tools\llama.cpp\build\bin\Release\llama-quantize.exe {export_dir / (ollama_tag + "-f16.gguf")} {gguf_path} Q4_K_M
   ```
   One-step alternative (larger file): `--outtype q8_0` on convert only.
3. Create Ollama model:
   ```bash
   cd {export_dir}
   ollama create {ollama_tag} -f Modelfile
   ```

## Adapter

- Version: {adapter_version}
- Path: {adapter_dir}
- Base: {base_model}
"""
    instructions_path.write_text(instructions, encoding="utf-8")

    register_model(
        {
            "ollama_tag": ollama_tag,
            "project": project,
            "version": adapter_version,
            "base_model": base_model,
            "adapter": f"projects/{project}/adapters/{adapter_version}",
            "gguf": str(gguf_path.relative_to(project_dir.parent.parent)) if gguf_path.exists() else None,
            "created_at": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
            "eval_score": None,
        },
        set_active=True,
    )

    print("\n".join(steps))
    print(f"Instructions: {instructions_path}")
    return modelfile_src


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--project", required=True)
    parser.add_argument("--adapter", default="v1")
    parser.add_argument("--quant", default="q4_k_m")
    parser.add_argument("--skip-merge", action="store_true")
    args = parser.parse_args()
    try:
        merge_and_export_gguf(args.project, args.adapter, args.quant, args.skip_merge)
        return 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
