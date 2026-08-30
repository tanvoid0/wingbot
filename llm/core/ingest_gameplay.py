#!/usr/bin/env python3
"""Capture gameplay sessions and manage review → approved workflow."""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from champion_names import load_champion_alias_map, normalize_champion_name, normalize_game_state  # noqa: E402
from jsonl_utils import load_jsonl  # noqa: E402
from project_loader import get_project_dir, project_path  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parent.parent.parent


def append_jsonl(path: Path, row: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as f:
        f.write(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n")


def resolve_champion(value: str | int | None, alias_map: dict[str, str]) -> str | None:
    if value is None:
        return None
    return normalize_champion_name(str(value), alias_map)


def snapshot_to_input(snapshot_data: dict, alias_map: dict[str, str]) -> dict | None:
    """Convert GameSnapshot or fixture envelope to compact game state."""
    data = snapshot_data.get("data", snapshot_data)
    champ_select = data.get("champion_select") or {}
    live = data.get("live_game") or {}

    my_champ = None
    my_role = "mid"
    enemies: list[str] = []

    if champ_select:
        my_team = champ_select.get("myTeam") or []
        their_team = champ_select.get("theirTeam") or []
        for p in my_team:
            if p.get("cellId") == champ_select.get("localPlayerCellId"):
                cid = p.get("championId") or p.get("championPickIntent")
                if cid:
                    my_champ = resolve_champion(cid, alias_map)
        for p in their_team:
            if p.get("championId"):
                resolved = resolve_champion(p["championId"], alias_map)
                if resolved:
                    enemies.append(resolved)

    if live and not my_champ:
        ap = live.get("activePlayer") or {}
        my_champ = resolve_champion(ap.get("championName"), alias_map) or my_champ
        for p in live.get("allPlayers") or []:
            if p.get("team") != ap.get("team"):
                if p.get("championName"):
                    resolved = resolve_champion(p["championName"], alias_map)
                    if resolved:
                        enemies.append(resolved)

    if not my_champ and not enemies:
        return None

    state = {
        "phase": "champ_select" if champ_select else "in_game",
        "my_champ": my_champ or "Unknown",
        "my_role": my_role,
        "enemies": enemies[:5],
        "queue": "ranked_solo",
        "patch": "15.12",
    }
    return normalize_game_state(state, alias_map)


def ingest_fixture(project: str, fixture_root: Path) -> int:
    project_dir = get_project_dir(project)
    alias_map = load_champion_alias_map(project_dir)
    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    raw_dir = project_dir / "datasets" / "gameplay" / "raw" / today
    raw_dir.mkdir(parents=True, exist_ok=True)

    count = 0
    for path in fixture_root.rglob("*.json"):
        try:
            envelope = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            continue
        inp = snapshot_to_input(envelope, alias_map)
        if not inp:
            continue
        row = {
            "captured_at": datetime.now(timezone.utc).isoformat(),
            "source": "wingbot_snapshot",
            "source_file": str(path.relative_to(fixture_root)),
            "input": inp,
            "model_output": "",
            "human_label": "pending",
            "human_correction": "",
            "patch": inp.get("patch", ""),
        }
        out = raw_dir / f"{path.stem}.capture.json"
        out.write_text(json.dumps(row, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        count += 1

    print(f"Ingested {count} fixtures -> {raw_dir}")
    return count


def iter_raw_captures(raw_root: Path) -> list[tuple[Path, dict]]:
    captures: list[tuple[Path, dict]] = []
    if not raw_root.exists():
        return captures
    for path in raw_root.rglob("*"):
        if path.suffix == ".json" and path.name.endswith(".capture.json"):
            try:
                captures.append((path, json.loads(path.read_text(encoding="utf-8"))))
            except json.JSONDecodeError:
                continue
        elif path.suffix == ".jsonl":
            for row in load_jsonl(path):
                captures.append((path, row))
    return captures


def review_pending(project: str, auto_approve_demo: bool = False) -> int:
    """List pending captures; optionally auto-approve for demo pipeline."""
    project_dir = get_project_dir(project)
    raw_root = project_dir / "datasets" / "gameplay" / "raw"
    approved_path = project_dir / "datasets" / "gameplay" / "approved.jsonl"

    approved = 0
    for path, row in iter_raw_captures(raw_root):
        if row.get("human_label") != "pending":
            continue
        print(f"\n--- {path.name} ---")
        print(f"Input: {json.dumps(row.get('input', {}), indent=2)}")
        if auto_approve_demo:
            row["human_label"] = "good"
            row["human_correction"] = row.get("human_correction") or ""
            append_jsonl(approved_path, {
                "messages": [
                    {"role": "user", "content": json.dumps(row["input"], separators=(",", ":"))},
                    {"role": "assistant", "content": row.get("human_correction") or "Play for objectives and track enemy jungler."},
                ],
                "source": row.get("source"),
                "captured_at": row.get("captured_at"),
            })
            row["human_label"] = "approved"
            path.write_text(json.dumps(row, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            approved += 1
        else:
            print("Run with --auto-approve-demo or edit raw files: set human_label to 'good' or add human_correction")

    print(f"Approved {approved} examples -> {approved_path}")
    return approved


def promote_reviewed(project: str) -> int:
    """Promote manually reviewed raw rows to approved.jsonl."""
    project_dir = get_project_dir(project)
    raw_root = project_dir / "datasets" / "gameplay" / "raw"
    approved_path = project_dir / "datasets" / "gameplay" / "approved.jsonl"
    count = 0

    for path, row in iter_raw_captures(raw_root):
        label = row.get("human_label", "")
        correction = (row.get("human_correction") or "").strip()
        if label == "good" or correction:
            content = correction or row.get("model_output") or "Stay focused on win condition."
            append_jsonl(approved_path, {
                "messages": [
                    {"role": "user", "content": json.dumps(row["input"], separators=(",", ":"))},
                    {"role": "assistant", "content": content},
                ],
                "source": row.get("source"),
                "captured_at": row.get("captured_at"),
            })
            row["human_label"] = "approved"
            count += 1
            if path.name.endswith(".capture.json"):
                path.write_text(json.dumps(row, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    print(f"Promoted {count} reviewed rows -> {approved_path}")
    return count


def main() -> int:
    parser = argparse.ArgumentParser(description="Ingest and review gameplay captures")
    parser.add_argument("--project", required=True)
    parser.add_argument("--from-fixture", type=Path, help="Ingest WingBot test-data/league fixtures")
    parser.add_argument("--review", action="store_true", help="Review pending captures")
    parser.add_argument("--promote", action="store_true", help="Promote good/edited rows to approved.jsonl")
    parser.add_argument("--auto-approve-demo", action="store_true")
    args = parser.parse_args()

    if args.from_fixture:
        ingest_fixture(args.project, args.from_fixture)
    if args.review:
        review_pending(args.project, auto_approve_demo=args.auto_approve_demo)
    if args.promote:
        promote_reviewed(args.project)
    if not any([args.from_fixture, args.review, args.promote]):
        parser.print_help()
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
