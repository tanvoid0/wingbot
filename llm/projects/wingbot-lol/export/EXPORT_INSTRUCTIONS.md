# Export instructions for wingbot-lol

Generated: 2026-06-15T22:48:00+00:00

## Steps completed

- Merged LoRA adapter into base weights at `export/merged/`
- F16 GGUF: `export/wingbot-lol-f16.gguf` (~7.8 GB)
- Q4_K_M GGUF: `export/wingbot-lol-q4_k_m.gguf` (~2.5 GB)
- Modelfile updated: `FROM ./wingbot-lol-q4_k_m.gguf`
- Ollama model `wingbot-lol` created from custom GGUF

## Re-export (if you retrain)

```powershell
cd D:\production\rust\wingbot\llm\core
.venv\Scripts\activate
python export_ollama.py --project wingbot-lol
```

Or manually:

1. Merge: `python export_ollama.py --project wingbot-lol` (no --skip-merge)
2. F16: `.venv\Scripts\python C:\tools\llama.cpp\convert_hf_to_gguf.py export\merged --outfile export\wingbot-lol-f16.gguf --outtype f16`
3. Quantize: `C:\tools\llama-bin\llama-quantize.exe export\wingbot-lol-f16.gguf export\wingbot-lol-q4_k_m.gguf Q4_K_M`
4. Create: `cd export && ollama create wingbot-lol -f Modelfile`

## Adapter

- Version: v1
- Path: D:\production\rust\wingbot\llm\projects\wingbot-lol\adapters\v1
- Base: google/gemma-3-4b-it
