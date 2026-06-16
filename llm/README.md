# WingBot LLM Pipeline

Reusable local LLM tooling for domain-specific models. First experiment: **League of Legends coaching** (`projects/wingbot-lol/`).

**Full guide:** [docs/CUSTOM_LLM_PLAYBOOK.md](../docs/CUSTOM_LLM_PLAYBOOK.md)  
**Status & checklist:** [docs/LLM_PIPELINE.md](../docs/LLM_PIPELINE.md)

## Quick Start (wingbot-lol)

```bash
cd llm/core
python -m venv .venv
.venv\Scripts\activate          # Windows
pip install -r requirements.txt

# 1. Extract knowledge
python ../projects/wingbot-lol/scripts/extract_ddragon.py
python ../projects/wingbot-lol/scripts/export_wingbot_rules.py

# 2. Merge + build dataset
python merge_knowledge.py --project wingbot-lol
python build_dataset.py --project wingbot-lol

# 3. Train (requires GPU + HuggingFace token)
python train_lora.py --project wingbot-lol

# 4. Export to Ollama
python export_ollama.py --project wingbot-lol

# 5. Evaluate
python eval.py --project wingbot-lol
```

## New Project

Repo defaults live in [`defaults.yaml`](defaults.yaml) (`base_model`, `ollama_base_model`). Each project must set `base_model` in its `project.yaml`; training scripts fail with fix instructions if it is missing.

```bash
cp -r projects/_template projects/my-app
# Edit projects/my-app/project.yaml (base_model required) and schemas/
python merge_knowledge.py --project my-app
python build_dataset.py --project my-app
```

## Layout

```
llm/
├── core/           # Shared scripts (--project <name>)
├── projects/       # Per-domain configs and data
├── registry.json   # Model versions and active tags
└── README.md
```
