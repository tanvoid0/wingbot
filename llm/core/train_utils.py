"""Shared training helpers — device, precision, BitsAndBytes config."""

from __future__ import annotations


def require_cuda() -> None:
    import torch

    if torch.cuda.is_available():
        name = torch.cuda.get_device_name(0)
        print(f"CUDA available: {name}")
        return

    raise RuntimeError(
        "CUDA GPU required for LoRA training, but torch.cuda.is_available() is False.\n"
        f"Installed torch: {torch.__version__}\n"
        "If you see '+cpu', PyTorch is the wrong build. Force reinstall CUDA torch:\n"
        "  pip uninstall torch -y\n"
        "  pip install torch --index-url https://download.pytorch.org/whl/cu128\n"
        "  # cu124 if cu128 fails on older GPUs\n"
        "Verify: python -c \"import torch; print(torch.__version__, torch.cuda.is_available())\""
    )


def resolve_precision() -> dict[str, bool]:
    """Pick bf16 or fp16 for SFTTrainer; never enable both."""
    import torch

    if not torch.cuda.is_available():
        return {"bf16": False, "fp16": False}

    if torch.cuda.is_bf16_supported():
        print("Training precision: bf16")
        return {"bf16": True, "fp16": False}

    print("Training precision: fp16 (bf16 not supported on this GPU)")
    return {"bf16": False, "fp16": True}


def bitsandbytes_config():
    import torch
    from transformers import BitsAndBytesConfig

    compute_dtype = (
        torch.bfloat16
        if torch.cuda.is_available() and torch.cuda.is_bf16_supported()
        else torch.float16
    )
    return BitsAndBytesConfig(
        load_in_4bit=True,
        bnb_4bit_quant_type="nf4",
        bnb_4bit_compute_dtype=compute_dtype,
        bnb_4bit_use_double_quant=True,
    )


def cuda_device_map() -> dict[str, int]:
    """Keep the full quantized model on GPU 0 — avoids bf16/CPU offload errors."""
    return {"": 0}
