#!/usr/bin/env python3
"""Download public LoL match data and convert to WingBot training examples.

Sources:
  hf-emerald    — BoostedJonP/league_of_legends_match_data (Emerald ranked CSV, ~25k rows)
  riot-matches  — Local match-v5 JSON from fetch_riot_matches.py (includes item builds)
  riot-cache    — Alias for riot-matches (legacy path)

Raw match stats do not include coaching labels. This script pairs lane matchups with
curated matchup knowledge and outcome heuristics to produce pseudo-labeled examples
in the compact game_state JSON format used by wingbot-lol.
"""

from __future__ import annotations

import argparse
import csv
import json
import sys
import urllib.request
from collections import defaultdict
from pathlib import Path

import jsonschema

sys.path.insert(0, str(Path(__file__).resolve().parent))
from build_dataset import advice_from_matchup, make_example  # noqa: E402
from champion_names import (  # noqa: E402
    load_champion_alias_map,
    normalize_champion_list,
    normalize_champion_name,
    normalize_game_state,
)
from jsonl_utils import load_jsonl, write_jsonl  # noqa: E402
from project_loader import get_project_dir, load_input_schema, load_project  # noqa: E402

LLM_ROOT = Path(__file__).resolve().parent.parent
REPO_ROOT = LLM_ROOT.parent

HF_EMERALD_URL = (
    "https://huggingface.co/datasets/BoostedJonP/league_of_legends_match_data/"
    "resolve/main/league_of_legends_emerald_match_data.csv"
)

ROLE_MAP = {
    "TOP": "top",
    "JUNGLE": "jungle",
    "MIDDLE": "mid",
    "MID": "mid",
    "BOTTOM": "adc",
    "ADC": "adc",
    "UTILITY": "support",
    "SUPPORT": "support",
}


def map_role(raw: str) -> str | None:
    return ROLE_MAP.get((raw or "").upper())


def build_matchup_index(matchups: list[dict]) -> dict[tuple[str, str, str], dict]:
    index: dict[tuple[str, str, str], dict] = {}
    for m in matchups:
        key = (m["your_champ"].lower(), m["enemy_champ"].lower(), m.get("role", "mid"))
        index[key] = m
    return index


def lookup_matchup(
    index: dict[tuple[str, str, str], dict],
    your_champ: str,
    enemy_champ: str,
    role: str,
) -> dict | None:
    direct = index.get((your_champ.lower(), enemy_champ.lower(), role))
    if direct:
        return direct
    return index.get((your_champ.lower(), enemy_champ.lower(), "mid"))


def load_item_names(project_dir: Path) -> dict[int, str]:
    """Map item ID → name from Data Dragon catalog."""
    item_path = project_dir / "knowledge" / "ddragon" / "item.json"
    if not item_path.exists():
        merged = project_dir / "knowledge" / "_merged" / "knowledge" / "ddragon" / "item.json"
        item_path = merged if merged.exists() else item_path
    if not item_path.exists():
        return {}
    try:
        data = json.loads(item_path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return {}
    names: dict[int, str] = {}
    for key, entry in (data.get("data") or {}).items():
        try:
            item_id = int(entry.get("id") or key)
        except (TypeError, ValueError):
            continue
        name = entry.get("name") or key
        if name and name != "Empty":
            names[item_id] = name
    return names


def participant_item_names(participant: dict, item_map: dict[int, str]) -> list[str]:
    items: list[str] = []
    for slot in range(7):
        raw = participant.get(f"item{slot}")
        if raw is None:
            continue
        try:
            item_id = int(raw)
        except (TypeError, ValueError):
            continue
        if item_id == 0:
            continue
        name = item_map.get(item_id)
        if name:
            items.append(name)
    return items


def item_build_note(player: dict, matchup: dict | None, won: bool) -> str:
    built = player.get("my_items") or []
    if not built:
        return ""
    built_str = ", ".join(built[:4])
    if won:
        return f"Winning build path: {built_str}."
    recommended = (matchup or {}).get("itemization") or []
    if recommended:
        missing = [i for i in recommended if i not in built]
        if missing:
            return f"Consider pivoting toward {', '.join(missing[:2])}; you finished {built_str}."
    return f"You completed {built_str} — evaluate if this fit the matchup."


def outcome_notes(row: dict, won: bool) -> str:
    notes: list[str] = []
    deaths = int(row.get("deaths") or 0)
    kills = int(row.get("kills") or 0)
    assists = int(row.get("assists") or 0)
    vision = float(row.get("vision_score") or 0)
    cs = int(row.get("total_minions_killed") or 0)
    duration = max(1, int(row.get("game_duration") or 1))
    cs_per_min = cs / (duration / 60)

    if won:
        notes.append("This pattern worked — repeat it next game.")
    else:
        notes.append("Adjust: reduce throw risk and play safer win conditions.")

    if deaths >= 8:
        notes.append("Too many deaths — ward more and respect ganks.")
    elif deaths <= 2 and kills + assists >= 8:
        notes.append("Strong impact — keep pressing with vision.")

    role = map_role(row.get("team_position", ""))
    if role == "support" and vision < 15:
        notes.append("Low vision — buy control wards before objectives.")
    elif role in ("adc", "mid", "top") and cs_per_min < 5:
        notes.append("Improve CS under pressure; don't coin-flip fights for plates.")

    return " ".join(notes)


def advice_for_player(
    player: dict,
    lane_opponent: dict | None,
    all_enemies: list[str],
    matchup_index: dict[tuple[str, str, str], dict],
) -> str:
    role = map_role(player.get("team_position", "")) or "mid"
    my_champ = player.get("champion_name") or ""
    enemy_name = (lane_opponent or {}).get("champion_name") or ""
    won = str(player.get("win", "")).upper() in ("TRUE", "1", "YES") or player.get("win") is True

    matchup = lookup_matchup(matchup_index, my_champ, enemy_name, role) if enemy_name else None
    if matchup:
        base = advice_from_matchup(matchup)
    elif enemy_name:
        base = f"Play {my_champ} into {enemy_name} for lane prio and objective setup."
    else:
        base = f"Group with team and play for {role} win condition."

    item_note = item_build_note(player, matchup, won)
    parts = [base]
    if item_note:
        parts.append(item_note)
    parts.append(outcome_notes(player, won))
    return " ".join(parts)


def normalize_player_row(row: dict, alias_map: dict[str, str]) -> dict:
    out = dict(row)
    if out.get("champion_name"):
        out["champion_name"] = normalize_champion_name(out["champion_name"], alias_map)
    if out.get("my_items"):
        out["my_items"] = list(out["my_items"])
    return out


def player_to_state(
    player: dict,
    lane_opponent: dict | None,
    all_enemies: list[str],
    alias_map: dict[str, str],
) -> dict:
    role = map_role(player.get("team_position", "")) or "mid"
    my_gold = int(player.get("gold_earned") or 0)
    opp_gold = int((lane_opponent or {}).get("gold_earned") or my_gold)
    duration = int(player.get("game_duration") or 1800)

    enemy_name = (lane_opponent or {}).get("champion_name")
    state = {
        "phase": "in_game",
        "my_champ": player.get("champion_name") or "Unknown",
        "my_role": role,
        "enemies": all_enemies[:5],
        "enemy_threats": [enemy_name] if enemy_name else [],
        "game_time_sec": duration,
        "gold_diff": my_gold - opp_gold,
        "my_items": list(player.get("my_items") or []),
        "queue": "ranked_solo",
        "patch": player.get("patch") or "15.12",
    }
    if role and lane_opponent and lane_opponent.get("champion_name"):
        state[f"enemy_{role}"] = lane_opponent["champion_name"]
    return normalize_game_state(state, alias_map)


def process_match_rows(
    rows: list[dict],
    matchup_index: dict[tuple[str, str, str], dict],
    schema: dict,
    alias_map: dict[str, str],
    *,
    require_matchup: bool = False,
) -> list[dict]:
    by_match: dict[str, list[dict]] = defaultdict(list)
    for row in rows:
        mid = row.get("match_id")
        if mid:
            by_match[mid].append(normalize_player_row(row, alias_map))

    examples: list[dict] = []
    for players in by_match.values():
        if len(players) < 2:
            continue

        teams: dict[int, list[dict]] = defaultdict(list)
        for p in players:
            try:
                teams[int(p.get("team_id", 0))].append(p)
            except (TypeError, ValueError):
                continue

        team_ids = [tid for tid in teams if teams[tid]]
        if len(team_ids) != 2:
            continue

        for tid in team_ids:
            enemy_tid = team_ids[1] if tid == team_ids[0] else team_ids[0]
            allies = teams[tid]
            enemies = teams[enemy_tid]
            enemy_names = normalize_champion_list(
                [e.get("champion_name") for e in enemies if e.get("champion_name")],
                alias_map,
            )

            by_role: dict[str, dict] = {}
            for e in enemies:
                r = map_role(e.get("team_position", ""))
                if r and r not in by_role:
                    by_role[r] = e

            for p in allies:
                role = map_role(p.get("team_position", ""))
                if not role or not p.get("champion_name"):
                    continue
                lane_opp = by_role.get(role)
                if require_matchup and lane_opp:
                    if not lookup_matchup(
                        matchup_index,
                        p["champion_name"],
                        lane_opp.get("champion_name", ""),
                        role,
                    ):
                        continue

                user = player_to_state(p, lane_opp, enemy_names, alias_map)
                try:
                    jsonschema.validate(user, schema)
                except jsonschema.ValidationError:
                    continue

                assistant = advice_for_player(p, lane_opp, enemy_names, matchup_index)
                examples.append(make_example(user, assistant))

    return examples


def download_hf_emerald(dest: Path) -> Path:
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.exists() and dest.stat().st_size > 1_000_000:
        print(f"Using cached {dest}")
        return dest

    print(f"Downloading {HF_EMERALD_URL} ...")
    req = urllib.request.Request(HF_EMERALD_URL, headers={"User-Agent": "wingbot/1.0"})
    with urllib.request.urlopen(req, timeout=180) as resp:
        dest.write_bytes(resp.read())
    print(f"Saved {dest.stat().st_size // 1024} KB -> {dest}")
    return dest


def load_csv_rows(path: Path, limit: int | None = None) -> list[dict]:
    rows: list[dict] = []
    with path.open(newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for row in reader:
            rows.append(row)
            if limit and len(rows) >= limit:
                break
    return rows


def load_riot_match_rows(cache_dir: Path, item_map: dict[int, str], limit: int | None = None) -> list[dict]:
    """Convert match-v5 JSON files into flat player rows with item builds."""
    rows: list[dict] = []
    if not cache_dir.exists():
        return rows

    for path in sorted(cache_dir.glob("*.json")):
        if path.name.endswith("_timeline.json"):
            continue
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            continue

        info = data.get("info") or data
        duration = info.get("gameDuration") or info.get("game_duration") or 0
        queue = info.get("queueId") or info.get("queue_id") or 420
        match_id = data.get("metadata", {}).get("matchId") or path.stem
        patch = (info.get("gameVersion") or "").split(".")[:2]
        patch_str = ".".join(patch) if patch else "15.12"

        for p in info.get("participants") or []:
            items = participant_item_names(p, item_map)
            rows.append({
                "match_id": match_id,
                "game_duration": duration,
                "queue_id": queue,
                "champion_name": p.get("championName") or p.get("champion_name"),
                "team_id": p.get("teamId") or p.get("team_id"),
                "team_position": p.get("teamPosition") or p.get("individualPosition") or p.get("role"),
                "win": p.get("win"),
                "kills": p.get("kills", 0),
                "deaths": p.get("deaths", 0),
                "assists": p.get("assists", 0),
                "gold_earned": p.get("goldEarned") or p.get("gold_earned") or 0,
                "total_minions_killed": (p.get("totalMinionsKilled") or 0)
                + (p.get("neutralMinionsKilled") or 0),
                "vision_score": p.get("visionScore") or p.get("vision_score") or 0,
                "my_items": items,
                "patch": patch_str,
            })
            if limit and len(rows) >= limit:
                return rows
    return rows


def load_riot_cache_rows(cache_dir: Path, limit: int | None = None) -> list[dict]:
    """Legacy loader without item map (empty items)."""
    return load_riot_match_rows(cache_dir, {}, limit=limit)


def dedupe_examples(examples: list[dict]) -> list[dict]:
    seen: set[str] = set()
    out: list[dict] = []
    for ex in examples:
        key = ex["messages"][0]["content"]
        if key not in seen:
            seen.add(key)
            out.append(ex)
    return out


def ingest_hf_emerald(
    project: str,
    *,
    limit: int | None = None,
    require_matchup: bool = False,
    max_examples: int | None = None,
) -> Path:
    project_dir = get_project_dir(project)
    schema = load_input_schema(project)
    matchups = load_jsonl(project_dir / "knowledge" / "matchups" / "matchups.jsonl")
    matchup_index = build_matchup_index(matchups)
    alias_map = load_champion_alias_map(project_dir)

    raw_dir = project_dir / "datasets" / "external" / "raw"
    csv_path = raw_dir / "hf_emerald_match_data.csv"
    download_hf_emerald(csv_path)

    rows = load_csv_rows(csv_path, limit=limit)
    print(f"Loaded {len(rows)} CSV rows")

    examples = process_match_rows(rows, matchup_index, schema, alias_map, require_matchup=require_matchup)
    examples = dedupe_examples(examples)
    if max_examples:
        examples = examples[:max_examples]

    out_path = project_dir / "datasets" / "external" / "hf_emerald.jsonl"
    write_jsonl(out_path, examples)
    print(f"Wrote {len(examples)} external examples -> {out_path}")
    return out_path


def ingest_riot_matches(
    project: str,
    cache_dir: Path,
    *,
    limit: int | None = None,
    require_matchup: bool = False,
    max_examples: int | None = None,
) -> Path:
    project_dir = get_project_dir(project)
    schema = load_input_schema(project)
    matchups = load_jsonl(project_dir / "knowledge" / "matchups" / "matchups.jsonl")
    matchup_index = build_matchup_index(matchups)
    alias_map = load_champion_alias_map(project_dir)
    item_map = load_item_names(project_dir)

    rows = load_riot_match_rows(cache_dir, item_map, limit=limit)
    print(f"Loaded {len(rows)} rows from Riot matches ({len(list(cache_dir.glob('*.json')))} files)")

    examples = process_match_rows(rows, matchup_index, schema, alias_map, require_matchup=require_matchup)
    examples = dedupe_examples(examples)
    if max_examples:
        examples = examples[:max_examples]

    out_path = project_dir / "datasets" / "external" / "riot_matches.jsonl"
    write_jsonl(out_path, examples)
    with_items = sum(1 for e in examples if json.loads(e["messages"][0]["content"]).get("my_items"))
    print(f"Wrote {len(examples)} external examples ({with_items} with items) -> {out_path}")
    return out_path


def ingest_riot_cache(
    project: str,
    cache_dir: Path,
    *,
    limit: int | None = None,
    require_matchup: bool = False,
    max_examples: int | None = None,
) -> Path:
    return ingest_riot_matches(
        project,
        cache_dir,
        limit=limit,
        require_matchup=require_matchup,
        max_examples=max_examples,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Ingest external LoL match datasets")
    parser.add_argument("--project", required=True)
    parser.add_argument(
        "--source",
        choices=["hf-emerald", "riot-matches", "riot-cache", "all"],
        default="hf-emerald",
        help="Data source to ingest",
    )
    parser.add_argument("--limit", type=int, help="Max CSV/cache rows to read")
    parser.add_argument("--max-examples", type=int, help="Cap output examples after processing")
    parser.add_argument(
        "--require-matchup",
        action="store_true",
        help="Only keep rows with a curated matchup in knowledge/matchups",
    )
    parser.add_argument(
        "--riot-cache-dir",
        type=Path,
        default=None,
        help="Directory with match-v5 JSON (default: datasets/external/raw/riot)",
    )
    args = parser.parse_args()
    riot_dir = args.riot_cache_dir or (get_project_dir(args.project) / "datasets" / "external" / "raw" / "riot")

    try:
        if args.source in ("hf-emerald", "all"):
            ingest_hf_emerald(
                args.project,
                limit=args.limit,
                require_matchup=args.require_matchup,
                max_examples=args.max_examples,
            )
        if args.source in ("riot-matches", "riot-cache", "all"):
            ingest_riot_matches(
                args.project,
                riot_dir,
                limit=args.limit,
                require_matchup=args.require_matchup,
                max_examples=args.max_examples,
            )
        return 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
