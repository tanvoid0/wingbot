#!/usr/bin/env python3
"""Build train.jsonl and eval.jsonl from project knowledge packs."""

from __future__ import annotations

import argparse
import json
import random
import sys
from pathlib import Path

import jsonschema

sys.path.insert(0, str(Path(__file__).resolve().parent))
from project_loader import get_project_dir, load_input_schema, load_project, project_path  # noqa: E402

LLM_ROOT = Path(__file__).resolve().parent.parent


def load_jsonl(path: Path) -> list[dict]:
    if not path.exists():
        return []
    rows = []
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if line:
            rows.append(json.loads(line))
    return rows


def make_example(user_obj: dict, assistant: str) -> dict:
    return {
        "messages": [
            {"role": "user", "content": json.dumps(user_obj, separators=(",", ":"))},
            {"role": "assistant", "content": assistant},
        ]
    }


def advice_from_matchup(m: dict, phase: str = "in_game") -> str:
    parts = []
    if m.get("lane_plan"):
        parts.append(m["lane_plan"])
    items = m.get("itemization") or []
    if items:
        parts.append(f"Itemize: {', '.join(items)}.")
    if m.get("win_condition"):
        parts.append(m["win_condition"])
    if m.get("playstyle"):
        parts.append(f"Playstyle: {m['playstyle'].replace('_', ' ')}.")
    return " ".join(parts) if parts else f"Play safe and scale against {m.get('enemy_champ', 'enemy')}."


def examples_from_matchups(matchups: list[dict], schema: dict) -> list[dict]:
    examples = []
    for m in matchups:
        role = m.get("role", "mid")
        phase = m.get("phase", "in_game")

        user_in_game = {
            "phase": "in_game",
            "my_champ": m["your_champ"],
            "my_role": role,
            "enemies": [m["enemy_champ"]],
            "enemy_threats": [m["enemy_champ"]],
            "game_time_sec": m.get("game_time_sec", 600),
            "gold_diff": m.get("gold_diff", 0),
            "my_items": m.get("my_items", []),
            "queue": m.get("queue", "ranked_solo"),
            "patch": m.get("patch", "15.12"),
        }
        user_draft = {
            "phase": "champ_select",
            "my_champ": m["your_champ"],
            "my_role": role,
            f"enemy_{role}": m["enemy_champ"],
            "enemies": [m["enemy_champ"]],
            "queue": "ranked_solo",
            "patch": m.get("patch", "15.12"),
        }

        try:
            jsonschema.validate(user_in_game, schema)
            examples.append(make_example(user_in_game, advice_from_matchup(m)))
        except jsonschema.ValidationError:
            pass

        try:
            jsonschema.validate(user_draft, schema)
            draft_advice = advice_from_matchup(m) or f"Draft {m['your_champ']} into {m['enemy_champ']} — play for lane prio."
            examples.append(make_example(user_draft, draft_advice))
        except jsonschema.ValidationError:
            pass

    return examples


def examples_from_item_counters(data: dict, schema: dict) -> list[dict]:
    examples = []
    for entry in data.get("champions", []):
        champ = entry.get("champion", "")
        items = entry.get("items", [])
        if not champ or not items:
            continue
        top = items[0]
        user = {
            "phase": "in_game",
            "my_champ": "Generic",
            "my_role": "top",
            "enemies": [champ],
            "enemy_threats": [champ],
            "game_time_sec": 900,
            "gold_diff": -200,
            "my_items": [],
            "queue": "ranked_solo",
            "patch": "15.12",
        }
        # Use a role-agnostic threat response
        user["my_champ"] = "Fighter"
        user["my_role"] = "top"
        assistant = f"Against {champ}, build {top['item']}: {top['reason']}."
        examples.append(make_example(user, assistant))

    for team in data.get("team_comp", []):
        cond = team.get("condition", "")
        for item in team.get("items", [])[:1]:
            user = {
                "phase": "in_game",
                "my_champ": "Ornn",
                "my_role": "top",
                "enemies": ["Jinx", "Caitlyn", "Yasuo"],
                "enemy_threats": [],
                "game_time_sec": 1200,
                "gold_diff": 0,
                "my_items": [],
                "queue": "ranked_solo",
                "patch": "15.12",
            }
            assistant = f"Team is {cond.replace('_', ' ')}. Prioritize {item['item']}: {item['reason']}."
            examples.append(make_example(user, assistant))

    return examples


def examples_from_edges(data: dict, schema: dict) -> list[dict]:
    examples = []
    for c in data.get("counters", []):
        user = {
            "phase": "champ_select",
            "my_champ": c["your_champ"],
            "my_role": "mid",
            "enemies": [c["enemy_champ"]],
            "queue": "ranked_solo",
            "patch": "15.12",
        }
        examples.append(
            make_example(user, f"Avoid unfavorable trades — {c['enemy_champ']} counters {c['your_champ']}. Ban or dodge if possible.")
        )
    for s in data.get("synergies", []):
        user = {
            "phase": "champ_select",
            "my_champ": s["your_champ"],
            "my_role": "mid",
            "allies": [s["ally_champ"]],
            "enemies": [],
            "queue": "ranked_solo",
            "patch": "15.12",
        }
        examples.append(
            make_example(user, f"Coordinate with {s['ally_champ']}: {s['note']}. Look for combo plays.")
        )
    return examples


def paraphrase_variants(examples: list[dict], factor: int = 2) -> list[dict]:
    """Duplicate with minor game_time / gold_diff variations to increase diversity."""
    out = list(examples)
    for ex in examples:
        try:
            user = json.loads(ex["messages"][0]["content"])
        except (json.JSONDecodeError, KeyError, IndexError):
            continue
        for i in range(factor - 1):
            variant = dict(user)
            if "game_time_sec" in variant:
                variant["game_time_sec"] = variant.get("game_time_sec", 600) + (i + 1) * 300
            if "gold_diff" in variant:
                variant["gold_diff"] = variant.get("gold_diff", 0) + (i + 1) * 400
            out.append(make_example(variant, ex["messages"][1]["content"]))
    return out


def build_dataset(project: str, eval_ratio: float = 0.1, seed: int = 42) -> tuple[Path, Path]:
    project_dir = get_project_dir(project)
    schema = load_input_schema(project)
    datasets_dir = project_dir / "datasets"
    datasets_dir.mkdir(parents=True, exist_ok=True)

    examples: list[dict] = []

    # Matchups
    matchups_path = project_dir / "knowledge" / "matchups" / "matchups.jsonl"
    matchups = load_jsonl(matchups_path)
    examples.extend(examples_from_matchups(matchups, schema))

    # Item counters
    item_path = project_dir / "knowledge" / "rules" / "item_counters.json"
    if item_path.exists():
        examples.extend(examples_from_item_counters(json.loads(item_path.read_text()), schema))

    # Champion edges
    edges_path = project_dir / "knowledge" / "rules" / "champion_edges.json"
    if edges_path.exists():
        examples.extend(examples_from_edges(json.loads(edges_path.read_text()), schema))

    # Shared decision patterns
    manifest = load_project(project)
    for shared in manifest.get("shared_datasets", []):
        shared_path = project_dir / shared
        if not shared_path.exists():
            shared_path = LLM_ROOT / "projects" / shared.lstrip("/")
        if shared_path.exists():
            examples.extend(load_jsonl(shared_path))

    examples = paraphrase_variants(examples, factor=2)

    random.seed(seed)
    random.shuffle(examples)
    split = max(1, int(len(examples) * eval_ratio))
    eval_set = examples[:split]
    train_set = examples[split:]

    train_path = datasets_dir / "train.jsonl"
    eval_path = datasets_dir / "eval.jsonl"

    train_path.write_text(
        "\n".join(json.dumps(e, ensure_ascii=False) for e in train_set) + ("\n" if train_set else ""),
        encoding="utf-8",
    )
    eval_path.write_text(
        "\n".join(json.dumps(e, ensure_ascii=False) for e in eval_set) + ("\n" if eval_set else ""),
        encoding="utf-8",
    )

    print(f"Built {len(train_set)} train, {len(eval_set)} eval examples")
    print(f"  -> {train_path}")
    print(f"  -> {eval_path}")
    return train_path, eval_path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--project", required=True)
    parser.add_argument("--eval-ratio", type=float, default=0.1)
    parser.add_argument("--seed", type=int, default=42)
    args = parser.parse_args()
    try:
        build_dataset(args.project, args.eval_ratio, args.seed)
        return 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
