# WingBot LLM Pipeline

League of Legends coaching model (`wingbot-lol`). **Local:** LoL knowledge ingest and dataset build. **Remote:** train, export, and eval via [agent-platform](https://github.com/) model-ops API (`/api/v1/model-ops/*`).

## Quick start

```bash
# 1. Ingest-only Python deps (no CUDA torch in wingbot)
npm run llm:setup

# 2. Build knowledge + train.jsonl locally
npm run llm:prepare

# 3. Ensure agent-platform is running (Docker :18410) and set token
#    AGENT_PLATFORM_MASTER_KEY=...  AGENT_PLATFORM_BASE_URL=http://127.0.0.1:18410

# 4. Sync artifacts + train (Docker GPU worker recommended)
npm run llm:sync
npm run llm:train
```

With agent-platform train profile running, `llm:train` runs LoRA + export in `agent-platform-train-1` and eval via API. Set `HF_TOKEN` in agent-platform `.env` for gated base models.

Full pipeline (prepare → sync → train):

```bash
npm run llm:wingbot-lol
```

## Commands

| npm script | Action |
|------------|--------|
| `llm:prepare` | Data Dragon, rules, matchups, Riot/HF ingest, `build_dataset.py` |
| `llm:sync` | Upload `project.yaml`, schemas, datasets, knowledge to agent-platform |
| `llm:train` | Sync + remote job: `train` → `export` → `eval` |
| `llm:eval` | Remote eval only |
| `llm:fetch-riot` | Bulk Riot match fetch + ingest |

## Layout

```
llm/
├── core/              # LoL ingest + build_dataset (domain-specific)
├── projects/wingbot-lol/
├── scripts/
│   ├── run-pipeline.mjs
│   └── agent-platform-client.mjs
└── README.md
```

Removed from wingbot (now on agent-platform): `train_lora.py`, `export_ollama.py`, `eval.py`, GPU deps.

Runtime inference still uses Ollama from the Tauri app (`ollama.model_name` → `wingbot-lol`).
