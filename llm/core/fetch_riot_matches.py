#!/usr/bin/env python3
"""Bulk-fetch ranked matches from Riot API for LLM training.

Seeds from Challenger / Grandmaster / Master league entries, then pulls recent
solo-queue match IDs per player and saves match-v5 JSON to disk.

Usage:
  set RIOT_API_KEY=RGAPI-...
  python fetch_riot_matches.py --project wingbot-lol --tier challenger --players 30 --matches-per-player 15

Then process:
  python ingest_external.py --project wingbot-lol --source riot-matches
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from project_loader import get_project_dir  # noqa: E402
from riot_client import RiotClient  # noqa: E402

RANKED_SOLO_QUEUE = 420


def riot_raw_dir(project: str) -> Path:
    return get_project_dir(project) / "datasets" / "external" / "raw" / "riot"


def save_json(path: Path, data: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False), encoding="utf-8")


def collect_puuids(client: RiotClient, tier: str, max_players: int) -> list[str]:
    entries = client.league_entries(tier)
    entries.sort(key=lambda e: int(e.get("leaguePoints") or 0), reverse=True)
    puuids: list[str] = []
    for entry in entries:
        puuid = entry.get("puuid")
        if puuid and puuid not in puuids:
            puuids.append(puuid)
        if len(puuids) >= max_players:
            break
    print(f"Collected {len(puuids)} {tier} PUUUIDs from {client.platform}")
    return puuids


def fetch_matches(
    project: str,
    *,
    tier: str = "challenger",
    players: int = 30,
    matches_per_player: int = 15,
    platform: str | None = None,
    routing: str | None = None,
    api_key: str | None = None,
    with_timeline: bool = False,
    skip_existing: bool = True,
) -> int:
    if api_key:
        client = RiotClient(api_key, platform=platform or "euw1", routing=routing)
    else:
        client = RiotClient.from_env_or_config(platform=platform or "euw1", routing=routing)

    out_dir = riot_raw_dir(project)
    out_dir.mkdir(parents=True, exist_ok=True)

    puuids = collect_puuids(client, tier, players)
    if not puuids:
        print("No league entries found.", file=sys.stderr)
        return 0

    seen_ids: set[str] = set()
    saved = 0
    skipped = 0

    for idx, puuid in enumerate(puuids, start=1):
        print(f"[{idx}/{len(puuids)}] Fetching match IDs for {puuid[:8]}...")
        try:
            ids = client.match_ids_for_puuid(puuid, count=matches_per_player)
        except RuntimeError as e:
            print(f"  skip: {e}", file=sys.stderr)
            continue

        for match_id in ids:
            if match_id in seen_ids:
                continue
            seen_ids.add(match_id)

            match_path = out_dir / f"{match_id}.json"
            if skip_existing and match_path.exists():
                skipped += 1
                continue

            try:
                match = client.match_by_id(match_id)
            except RuntimeError as e:
                print(f"  skip match {match_id}: {e}", file=sys.stderr)
                continue

            queue = (match.get("info") or {}).get("queueId")
            if queue != RANKED_SOLO_QUEUE:
                continue

            save_json(match_path, match)
            saved += 1

            if with_timeline:
                timeline_path = out_dir / f"{match_id}_timeline.json"
                if not (skip_existing and timeline_path.exists()):
                    try:
                        timeline = client.timeline_by_id(match_id)
                        save_json(timeline_path, timeline)
                    except RuntimeError as e:
                        print(f"  timeline skip {match_id}: {e}", file=sys.stderr)

            if saved % 25 == 0:
                print(f"  saved {saved} matches so far...")

    print(f"Done. Saved {saved} new matches, skipped {skipped} existing -> {out_dir}")
    return saved


def main() -> int:
    parser = argparse.ArgumentParser(description="Bulk-fetch Riot match-v5 data for training")
    parser.add_argument("--project", required=True)
    parser.add_argument("--tier", choices=["challenger", "grandmaster", "master"], default="challenger")
    parser.add_argument("--players", type=int, default=30, help="League entries to seed from")
    parser.add_argument("--matches-per-player", type=int, default=15)
    parser.add_argument("--platform", help="Platform routing (default: from WingBot config or euw1)")
    parser.add_argument("--routing", help="Regional routing (americas/europe/asia/sea)")
    parser.add_argument("--api-key", help="Riot API key (default: RIOT_API_KEY env or WingBot config)")
    parser.add_argument("--with-timeline", action="store_true", help="Also fetch match timelines")
    parser.add_argument("--force", action="store_true", help="Re-download even if cached")
    args = parser.parse_args()

    try:
        fetch_matches(
            args.project,
            tier=args.tier,
            players=args.players,
            matches_per_player=args.matches_per_player,
            platform=args.platform,
            routing=args.routing,
            api_key=args.api_key,
            with_timeline=args.with_timeline,
            skip_existing=not args.force,
        )
        return 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
