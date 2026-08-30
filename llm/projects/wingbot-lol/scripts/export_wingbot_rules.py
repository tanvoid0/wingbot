#!/usr/bin/env python3
"""Export item counter and champion edge rules from WingBot Rust sources."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_DIR = SCRIPT_DIR.parent
REPO_ROOT = PROJECT_DIR.parent.parent.parent
RULES_DIR = PROJECT_DIR / "knowledge" / "rules"

ITEM_ANALYZER = REPO_ROOT / "src-tauri" / "src" / "data_processing" / "item_analyzer.rs"
GAME_RS = REPO_ROOT / "src-tauri" / "src" / "game.rs"


def parse_item_counters(source: str) -> tuple[list[dict], list[dict]]:
    """Extract champion -> item mappings from item_analyzer.rs match arms."""
    counters_map: dict[str, list[dict]] = {}

    arm_pattern = re.compile(
        r'"(?P<champs>[^"]+)"(?:\s*\|\s*"(?P<champ2>[^"]+)")?\s*=>\s*\{',
    )
    json_block = re.compile(
        r'\{\s*"item":\s*"([^"]+)"\s*,\s*"reason":\s*"([^"]*)"\s*,\s*"priority":\s*"([^"]+)"\s*\}',
    )

    positions = [(m.start(), m.group("champs"), m.group("champ2")) for m in arm_pattern.finditer(source)]
    for idx, (start, c1, c2) in enumerate(positions):
        end = positions[idx + 1][0] if idx + 1 < len(positions) else len(source)
        block = source[start:end]
        champs = [c1]
        if c2:
            champs.append(c2)
        items = [
            {"item": item, "reason": reason, "priority": priority}
            for item, reason, priority in json_block.findall(block)
        ]
        if not items:
            continue
        for champ in champs:
            key = champ.title() if champ.islower() else champ
            if key in ("Tank", "Assassin", "Mage"):
                counters_map[key] = items
            elif key not in counters_map:
                counters_map[key] = items

    counters = [{"champion": k, "items": v} for k, v in counters_map.items()]
    team_rules: list[dict] = []
    if "ad_champions >= 3" in source:
        team_rules.append({
            "condition": "ad_heavy",
            "description": "3+ AD champions on enemy team",
            "items": [
                {"item": "Thornmail", "reason": "Reflects damage against AD-heavy teams", "priority": "high"},
                {"item": "Randuin's Omen", "reason": "Reduces critical strike damage", "priority": "high"},
            ],
        })
    if "ap_champions >= 3" in source:
        team_rules.append({
            "condition": "ap_heavy",
            "description": "3+ AP champions on enemy team",
            "items": [
                {"item": "Spirit Visage", "reason": "Magic resist and healing", "priority": "high"},
                {"item": "Adaptive Helm", "reason": "Reduces repeated magic damage", "priority": "medium"},
            ],
        })
    if "assassin_champions >= 2" in source:
        team_rules.append({
            "condition": "assassin_heavy",
            "description": "2+ assassins on enemy team",
            "items": [
                {"item": "Guardian Angel", "reason": "Protection against multiple assassins", "priority": "high"},
            ],
        })

    return counters, team_rules


def parse_champion_edges(source: str) -> dict:
    """Extract counter and synergy pairs from game.rs match arms."""
    counters: list[dict] = []
    synergies: list[dict] = []

    pair_pattern = re.compile(
        r'\("([^"]+)",\s*"([^"]+)"\)\s*\|\s*\("([^"]+)",\s*"([^"]+)"\)\s*=>\s*true'
    )

    def extract_block(fn_name: str, next_fn: str | None = None) -> str:
        if fn_name not in source:
            return ""
        start = source.index(fn_name)
        end = len(source)
        if next_fn and next_fn in source[start:]:
            end = start + source[start:].index(next_fn)
        return source[start:end]

    counter_block = extract_block("fn is_champion_countered_by", "fn has_champion_synergy_with")
    synergy_block = extract_block("fn has_champion_synergy_with", "\n}\n")

    for m in pair_pattern.finditer(counter_block):
        for a, b in [(m.group(1), m.group(2)), (m.group(3), m.group(4))]:
            counters.append({
                "your_champ": a.title(),
                "enemy_champ": b.title(),
                "type": "counter",
                "note": f"{a.title()} is countered by {b.title()}",
            })

    for m in pair_pattern.finditer(synergy_block):
        for a, b in [(m.group(1), m.group(2)), (m.group(3), m.group(4))]:
            synergies.append({
                "your_champ": a.title(),
                "ally_champ": b.title(),
                "type": "synergy",
                "note": f"{a.title()} synergizes with {b.title()}",
            })

    return {"counters": counters, "synergies": synergies}


def main() -> int:
    RULES_DIR.mkdir(parents=True, exist_ok=True)

    if not ITEM_ANALYZER.exists():
        print(f"Missing {ITEM_ANALYZER}", file=sys.stderr)
        return 1
    if not GAME_RS.exists():
        print(f"Missing {GAME_RS}", file=sys.stderr)
        return 1

    item_source = ITEM_ANALYZER.read_text(encoding="utf-8")
    game_source = GAME_RS.read_text(encoding="utf-8")

    champ_counters, team_rules = parse_item_counters(item_source)
    edges = parse_champion_edges(game_source)

    (RULES_DIR / "item_counters.json").write_text(
        json.dumps({"champions": champ_counters, "team_comp": team_rules}, indent=2),
        encoding="utf-8",
    )
    (RULES_DIR / "champion_edges.json").write_text(
        json.dumps(edges, indent=2), encoding="utf-8"
    )

    print(f"Exported {len(champ_counters)} champion item rules")
    print(f"Exported {len(edges['counters'])} counter edges, {len(edges['synergies'])} synergy edges")
    print(f"Exported {len(team_rules)} team comp rules")
    return 0


if __name__ == "__main__":
    sys.exit(main())
