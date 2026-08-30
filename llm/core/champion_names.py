#!/usr/bin/env python3
"""Normalize champion identifiers to Data Dragon display names."""

from __future__ import annotations

import json
from functools import lru_cache
from pathlib import Path


def _champion_catalog_paths(project_dir: Path) -> list[Path]:
    return [
        project_dir / "knowledge" / "ddragon" / "champion.json",
        project_dir / "knowledge" / "_merged" / "knowledge" / "ddragon" / "champion.json",
    ]


@lru_cache(maxsize=8)
def _load_alias_map(catalog_path: str) -> dict[str, str]:
    path = Path(catalog_path)
    if not path.exists():
        return {}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return {}

    alias_map: dict[str, str] = {}
    for entry in (data.get("data") or {}).values():
        display = entry.get("name") or entry.get("id")
        if not display:
            continue
        aliases = {
            entry.get("id"),
            entry.get("name"),
            entry.get("key"),
        }
        for alias in aliases:
            if alias is not None:
                alias_map[str(alias).strip()] = display
                alias_map[str(alias).strip().lower()] = display
    return alias_map


def load_champion_alias_map(project_dir: Path) -> dict[str, str]:
    for path in _champion_catalog_paths(project_dir):
        if path.exists():
            return _load_alias_map(str(path.resolve()))
    return {}


def normalize_champion_name(name: str | None, alias_map: dict[str, str]) -> str:
    if not name:
        return "Unknown"
    raw = str(name).strip()
    if not raw:
        return "Unknown"
    if raw in alias_map:
        return alias_map[raw]
    lower = raw.lower()
    if lower in alias_map:
        return alias_map[lower]
    return raw


def normalize_champion_list(names: list[str] | None, alias_map: dict[str, str]) -> list[str]:
    if not names:
        return []
    out: list[str] = []
    for name in names:
        normalized = normalize_champion_name(name, alias_map)
        if normalized not in out:
            out.append(normalized)
    return out


def normalize_game_state(state: dict, alias_map: dict[str, str]) -> dict:
    """Normalize champion fields inside a compact game_state object."""
    normalized = dict(state)
    if "my_champ" in normalized:
        normalized["my_champ"] = normalize_champion_name(normalized.get("my_champ"), alias_map)
    for key in ("enemies", "allies", "enemy_threats", "my_items"):
        if key in normalized and isinstance(normalized[key], list):
            if key == "my_items":
                continue
            normalized[key] = normalize_champion_list(normalized[key], alias_map)
    for key in ("enemy_top", "enemy_jungle", "enemy_mid", "enemy_adc", "enemy_support"):
        if key in normalized:
            normalized[key] = normalize_champion_name(normalized.get(key), alias_map)
    return normalized
