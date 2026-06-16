#!/usr/bin/env python3
"""Fetch Data Dragon champion and item catalogs into knowledge/ddragon/."""

from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from pathlib import Path

import requests

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_DIR = SCRIPT_DIR.parent
KNOWLEDGE_DIR = PROJECT_DIR / "knowledge" / "ddragon"
CHAMPIONS_DIR = KNOWLEDGE_DIR / "champions"
ITEMS_DIR = KNOWLEDGE_DIR / "items"

VERSIONS_URL = "https://ddragon.leagueoflegends.com/api/versions.json"
CDN_BASE = "https://ddragon.leagueoflegends.com/cdn"


def fetch_json(url: str) -> dict | list:
    resp = requests.get(url, timeout=60)
    resp.raise_for_status()
    return resp.json()


def main() -> int:
    KNOWLEDGE_DIR.mkdir(parents=True, exist_ok=True)
    CHAMPIONS_DIR.mkdir(parents=True, exist_ok=True)
    ITEMS_DIR.mkdir(parents=True, exist_ok=True)

    versions = fetch_json(VERSIONS_URL)
    patch = versions[0]
    print(f"Using Data Dragon patch: {patch}")

    champ_url = f"{CDN_BASE}/{patch}/data/en_US/champion.json"
    item_url = f"{CDN_BASE}/{patch}/data/en_US/item.json"

    champion_data = fetch_json(champ_url)
    item_data = fetch_json(item_url)

    # Full catalogs
    (KNOWLEDGE_DIR / "champion.json").write_text(
        json.dumps(champion_data, indent=2), encoding="utf-8"
    )
    (KNOWLEDGE_DIR / "item.json").write_text(
        json.dumps(item_data, indent=2), encoding="utf-8"
    )

    # Per-champion files for RAG / dataset generation
    for key, champ in champion_data.get("data", {}).items():
        slim = {
            "id": champ.get("key"),
            "name": champ.get("name"),
            "title": champ.get("title"),
            "tags": champ.get("tags", []),
            "partype": champ.get("partype"),
            "info": champ.get("info"),
            "stats": champ.get("stats"),
            "blurb": (champ.get("blurb") or "")[:500],
        }
        (CHAMPIONS_DIR / f"{key.lower()}.json").write_text(
            json.dumps(slim, indent=2), encoding="utf-8"
        )

    for item_id, item in item_data.get("data", {}).items():
        if item_id == "0":
            continue
        slim = {
            "id": item_id,
            "name": item.get("name"),
            "plaintext": item.get("plaintext"),
            "description": (item.get("description") or "")[:800],
            "gold": item.get("gold"),
            "tags": item.get("tags", []),
            "stats": item.get("stats"),
        }
        (ITEMS_DIR / f"{item_id}.json").write_text(
            json.dumps(slim, indent=2), encoding="utf-8"
        )

    meta = {
        "patch": patch,
        "fetched_at": datetime.now(timezone.utc).isoformat(),
        "champion_count": len(champion_data.get("data", {})),
        "item_count": len(item_data.get("data", {})) - 1,
        "source": "https://ddragon.leagueoflegends.com",
    }
    (KNOWLEDGE_DIR / "meta.json").write_text(
        json.dumps(meta, indent=2), encoding="utf-8"
    )

    print(f"Wrote {meta['champion_count']} champions, {meta['item_count']} items")
    return 0


if __name__ == "__main__":
    sys.exit(main())
