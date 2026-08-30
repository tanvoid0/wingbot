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
from champion_names import load_champion_alias_map, normalize_game_state  # noqa: E402
from jsonl_utils import load_jsonl, write_jsonl  # noqa: E402
from project_loader import get_project_dir, load_input_schema, load_project, project_path  # noqa: E402

LLM_ROOT = Path(__file__).resolve().parent.parent


def normalize_training_example(example: dict, alias_map: dict[str, str]) -> dict:
    messages = example.get("messages")
    if not isinstance(messages, list) or len(messages) < 2:
        return example
    try:
        user_obj = json.loads(messages[0]["content"])
    except (json.JSONDecodeError, KeyError, TypeError):
        return example
    normalized = normalize_game_state(user_obj, alias_map)
    return make_example(normalized, messages[1]["content"])


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
    role_defaults = {
        "Yasuo": ("top", "Aatrox"),
        "Yone": ("top", "Aatrox"),
        "Fiora": ("top", "Ornn"),
        "Camille": ("top", "Ornn"),
        "Katarina": ("mid", "Syndra"),
        "Akali": ("mid", "Syndra"),
        "Zed": ("mid", "Lux"),
        "Talon": ("mid", "Lux"),
        "Tank": ("adc", "Ashe"),
        "Assassin": ("adc", "Jinx"),
        "Mage": ("top", "Malphite"),
    }
    for entry in data.get("champions", []):
        threat = entry.get("champion", "")
        items = entry.get("items", [])
        if not threat or not items:
            continue
        top = items[0]
        reason = top.get("reason") or "Counters their damage profile"
        role, my_champ = role_defaults.get(threat, ("top", "Fighter"))
        user = {
            "phase": "in_game",
            "my_champ": my_champ,
            "my_role": role,
            "enemies": [threat] if threat not in ("Tank", "Assassin", "Mage") else ["Malphite"],
            "enemy_threats": [threat] if threat not in ("Tank", "Assassin", "Mage") else ["Malphite"],
            "game_time_sec": 900,
            "gold_diff": -200,
            "my_items": [],
            "queue": "ranked_solo",
            "patch": "15.12",
        }
        if threat == "Tank":
            user["enemies"] = ["Ornn", "Malphite"]
            user["enemy_threats"] = ["Ornn"]
        elif threat == "Assassin":
            user["enemies"] = ["Zed", "Kha'Zix"]
            user["enemy_threats"] = ["Zed", "Kha'Zix"]
        elif threat == "Mage":
            user["enemies"] = ["Syndra", "Lux", "Brand"]
            user["enemy_threats"] = ["Syndra"]
        assistant = f"Against {threat}, build {top['item']}: {reason}."
        examples.append(make_example(user, assistant))

    team_enemies = {
        "ad_heavy": ["Yasuo", "Yone", "Jinx", "Draven", "Lee Sin"],
        "ap_heavy": ["Syndra", "Lux", "Brand", "Elise", "Karma"],
        "assassin_heavy": ["Zed", "Kha'Zix", "Talon", "Jinx", "Nautilus"],
    }
    for team in data.get("team_comp", []):
        cond = team.get("condition", "")
        enemies = team_enemies.get(cond, ["Jinx", "Caitlyn", "Yasuo"])
        for item in team.get("items", [])[:1]:
            user = {
                "phase": "in_game",
                "my_champ": "Ornn",
                "my_role": "top",
                "enemies": enemies,
                "enemy_threats": enemies[:2],
                "game_time_sec": 1200,
                "gold_diff": 0,
                "my_items": [],
                "queue": "ranked_solo",
                "patch": "15.12",
            }
            reason = item.get("reason") or "Counters team composition"
            assistant = f"Team is {cond.replace('_', ' ')}. Prioritize {item['item']}: {reason}."
            examples.append(make_example(user, assistant))

    return examples


def examples_from_edges(data: dict, schema: dict) -> list[dict]:
    role_map = {
        "Yasuo": "mid", "Katarina": "mid", "Jinx": "adc", "Master Yi": "jungle",
        "Malphite": "top", "Gragas": "jungle", "Lulu": "support", "Kog'Maw": "adc",
    }
    examples = []
    for c in data.get("counters", []):
        role = role_map.get(c["your_champ"], "mid")
        user = {
            "phase": "champ_select",
            "my_champ": c["your_champ"],
            "my_role": role,
            "enemies": [c["enemy_champ"]],
            f"enemy_{role}": c["enemy_champ"],
            "queue": "ranked_solo",
            "patch": "15.12",
        }
        examples.append(
            make_example(user, f"Avoid unfavorable trades — {c['enemy_champ']} counters {c['your_champ']}. Ban or dodge if possible.")
        )
    for s in data.get("synergies", []):
        role = role_map.get(s["your_champ"], "mid")
        user = {
            "phase": "champ_select",
            "my_champ": s["your_champ"],
            "my_role": role,
            "allies": [s["ally_champ"]],
            "enemies": [],
            "queue": "ranked_solo",
            "patch": "15.12",
        }
        examples.append(
            make_example(user, f"Coordinate with {s['ally_champ']}: {s['note']}. Look for combo plays.")
        )
    return examples


def examples_from_situational_variants(matchups: list[dict], schema: dict) -> list[dict]:
    """Early/mid/late timing and gold-state variants for richer in-game coverage."""
    examples = []
    time_states = [
        (360, 0, "Early game — respect level 2-3 power spikes."),
        (1200, 0, "Mid game — contest vision before objectives."),
        (1800, 0, "Late game — play for Baron or elder setups."),
    ]
    gold_states = [
        (-800, "You are behind — reduce risk and farm safely."),
        (1000, "You are ahead — press advantage and zone resources."),
    ]

    for m in matchups:
        role = m.get("role", "mid")
        base_advice = advice_from_matchup(m)
        for game_time, gold_diff, timing_note in time_states:
            user = {
                "phase": "in_game",
                "my_champ": m["your_champ"],
                "my_role": role,
                "enemies": [m["enemy_champ"]],
                "enemy_threats": [m["enemy_champ"]],
                "game_time_sec": game_time,
                "gold_diff": gold_diff,
                "my_items": m.get("itemization", [])[:1],
                "queue": "ranked_solo",
                "patch": m.get("patch", "15.12"),
            }
            try:
                jsonschema.validate(user, schema)
                assistant = f"{timing_note} {base_advice}"
                examples.append(make_example(user, assistant))
            except jsonschema.ValidationError:
                pass

        for gold_diff, gold_note in gold_states:
            user = {
                "phase": "in_game",
                "my_champ": m["your_champ"],
                "my_role": role,
                "enemies": [m["enemy_champ"]],
                "enemy_threats": [m["enemy_champ"]],
                "game_time_sec": 900,
                "gold_diff": gold_diff,
                "my_items": [],
                "queue": "ranked_solo",
                "patch": m.get("patch", "15.12"),
            }
            try:
                jsonschema.validate(user, schema)
                assistant = f"{gold_note} {base_advice}"
                examples.append(make_example(user, assistant))
            except jsonschema.ValidationError:
                pass

    return examples


def examples_from_full_draft(matchups: list[dict], schema: dict) -> list[dict]:
    """Champ select with multiple revealed enemies."""
    examples = []
    sample_comps = [
        ["Jinx", "Thresh", "Lee Sin", "Syndra", "Malphite"],
        ["Draven", "Leona", "Graves", "Ahri", "Ornn"],
        ["Caitlyn", "Morgana", "Vi", "Orianna", "Sion"],
    ]
    for i, m in enumerate(matchups[: min(len(matchups), len(sample_comps) * 3)]):
        role = m.get("role", "mid")
        enemies = list(sample_comps[i % len(sample_comps)])
        if m["enemy_champ"] not in enemies:
            enemies[0] = m["enemy_champ"]
        user = {
            "phase": "champ_select",
            "my_champ": m["your_champ"],
            "my_role": role,
            "enemies": enemies,
            f"enemy_{role}": m["enemy_champ"],
            "queue": "ranked_solo",
            "patch": m.get("patch", "15.12"),
        }
        try:
            jsonschema.validate(user, schema)
            draft_advice = advice_from_matchup(m)
            assistant = f"Focus {m['enemy_champ']} in lane. {draft_advice}"
            examples.append(make_example(user, assistant))
        except jsonschema.ValidationError:
            pass
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
    alias_map = load_champion_alias_map(project_dir)

    examples: list[dict] = []

    # Matchups
    matchups_path = project_dir / "knowledge" / "matchups" / "matchups.jsonl"
    matchups = load_jsonl(matchups_path)
    examples.extend(examples_from_matchups(matchups, schema))

    # Item counters
    item_path = project_dir / "knowledge" / "rules" / "item_counters.json"
    if item_path.exists():
        examples.extend(examples_from_item_counters(json.loads(item_path.read_text()), schema))

    # Champion edges (exported from Rust + optional supplement)
    edges_path = project_dir / "knowledge" / "rules" / "champion_edges.json"
    if edges_path.exists():
        edges_data = json.loads(edges_path.read_text())
        supplement_path = project_dir / "knowledge" / "rules" / "champion_edges_supplement.json"
        if supplement_path.exists():
            supplement = json.loads(supplement_path.read_text())
            edges_data["counters"].extend(supplement.get("counters", []))
            edges_data["synergies"].extend(supplement.get("synergies", []))
        examples.extend(examples_from_edges(edges_data, schema))

    # Situational variants (time/gold states, multi-enemy draft)
    examples.extend(examples_from_situational_variants(matchups, schema))
    examples.extend(examples_from_full_draft(matchups, schema))

    # Shared decision patterns
    manifest = load_project(project)
    for shared in manifest.get("shared_datasets", []):
        shared_path = project_dir / shared
        if not shared_path.exists():
            shared_path = LLM_ROOT / "projects" / shared.lstrip("/")
        if shared_path.exists():
            examples.extend(load_jsonl(shared_path))

    # External match-derived datasets (ingest_external.py)
    for ext in manifest.get("external_datasets", []):
        ext_path = project_dir / ext
        if ext_path.exists():
            external = load_jsonl(ext_path)
            examples.extend(external)
            print(f"  + {len(external)} external examples from {ext_path.name}")

    examples = [normalize_training_example(ex, alias_map) for ex in examples]

    # Deduplicate by user content (keep first)
    seen_users: set[str] = set()
    deduped: list[dict] = []
    for ex in examples:
        key = ex["messages"][0]["content"]
        if key not in seen_users:
            seen_users.add(key)
            deduped.append(ex)
    examples = deduped

    examples = paraphrase_variants(examples, factor=2)

    random.seed(seed)
    random.shuffle(examples)
    split = max(1, int(len(examples) * eval_ratio))
    eval_set = examples[:split]
    train_set = examples[split:]

    train_path = datasets_dir / "train.jsonl"
    eval_path = datasets_dir / "eval.jsonl"

    write_jsonl(train_path, train_set)
    write_jsonl(eval_path, eval_set)

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
