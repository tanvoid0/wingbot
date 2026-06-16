"""Shared utilities for loading LLM project manifests."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import yaml

LLM_ROOT = Path(__file__).resolve().parent.parent
PROJECTS_DIR = LLM_ROOT / "projects"
REGISTRY_PATH = LLM_ROOT / "registry.json"
DEFAULTS_PATH = LLM_ROOT / "defaults.yaml"

_defaults_cache: dict[str, Any] | None = None


def load_defaults() -> dict[str, Any]:
    """Load repo-wide LLM defaults from llm/defaults.yaml."""
    global _defaults_cache
    if _defaults_cache is not None:
        return _defaults_cache

    if not DEFAULTS_PATH.exists():
        raise FileNotFoundError(
            f"Missing {DEFAULTS_PATH}. Restore llm/defaults.yaml with at least:\n"
            "  base_model: <huggingface-model-id>\n"
            "  ollama_base_model: <ollama-tag>"
        )

    with DEFAULTS_PATH.open(encoding="utf-8") as f:
        data = yaml.safe_load(f) or {}

    if not isinstance(data, dict):
        raise ValueError(f"Invalid {DEFAULTS_PATH}: expected a YAML mapping at the top level.")

    _defaults_cache = data
    return data


def require_base_model(manifest: dict[str, Any], project: str | None = None) -> str:
    """Return the project's HuggingFace base_model or fail with fix instructions."""
    base = manifest.get("base_model")
    if isinstance(base, str) and base.strip():
        return base.strip()

    project_name = project or manifest.get("_name") or manifest.get("name") or "<project>"
    manifest_path = PROJECTS_DIR / project_name / "project.yaml"
    defaults = load_defaults()
    example = defaults.get("base_model", "<huggingface-model-id>")

    raise ValueError(
        f"Missing required 'base_model' in {manifest_path}.\n"
        f"Add a HuggingFace model ID, e.g.:\n"
        f"  base_model: {example}\n"
        f"See {DEFAULTS_PATH} for the current default."
    )


def get_ollama_base_model() -> str:
    """Return the default Ollama base tag from llm/defaults.yaml."""
    defaults = load_defaults()
    model = defaults.get("ollama_base_model")
    if isinstance(model, str) and model.strip():
        return model.strip()

    raise ValueError(
        f"Missing required 'ollama_base_model' in {DEFAULTS_PATH}.\n"
        "Add an Ollama model tag, e.g.:\n"
        "  ollama_base_model: gemma4:latest"
    )


def get_project_dir(project: str) -> Path:
    path = PROJECTS_DIR / project
    if not path.is_dir():
        raise FileNotFoundError(f"Project not found: {project} ({path})")
    return path


def load_project(project: str) -> dict[str, Any]:
    project_dir = get_project_dir(project)
    manifest_path = project_dir / "project.yaml"
    if not manifest_path.exists():
        raise FileNotFoundError(f"Missing project.yaml: {manifest_path}")
    with manifest_path.open(encoding="utf-8") as f:
        data = yaml.safe_load(f)
    data["_project_dir"] = str(project_dir)
    data["_name"] = project
    return data


def project_path(project: str, *parts: str) -> Path:
    return get_project_dir(project).joinpath(*parts)


def load_input_schema(project: str) -> dict[str, Any]:
    manifest = load_project(project)
    schema_rel = manifest.get("input_schema", "schemas/input.schema.json")
    schema_path = project_path(project, schema_rel)
    with schema_path.open(encoding="utf-8") as f:
        return json.load(f)


def load_registry() -> dict[str, Any]:
    if not REGISTRY_PATH.exists():
        return {"models": [], "active": {}}
    with REGISTRY_PATH.open(encoding="utf-8") as f:
        return json.load(f)


def save_registry(data: dict[str, Any]) -> None:
    with REGISTRY_PATH.open("w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)
        f.write("\n")


def register_model(entry: dict[str, Any], set_active: bool = True) -> None:
    registry = load_registry()
    tag = entry["ollama_tag"]
    registry["models"] = [m for m in registry["models"] if m.get("ollama_tag") != tag]
    registry["models"].append(entry)
    if set_active:
        project = entry.get("project", entry["ollama_tag"])
        registry.setdefault("active", {})[project] = tag
    save_registry(registry)
