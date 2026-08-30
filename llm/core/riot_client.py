#!/usr/bin/env python3
"""Minimal Riot API client for LLM dataset fetching."""

from __future__ import annotations

import json
import os
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

PLATFORM_TO_ROUTING = {
    "na1": "americas",
    "br1": "americas",
    "la1": "americas",
    "la2": "americas",
    "euw1": "europe",
    "eun1": "europe",
    "tr1": "europe",
    "kr": "asia",
    "jp1": "asia",
    "oc1": "sea",
}

LEAGUE_QUEUES = {
    "challenger": "RANKED_SOLO_5x5",
    "grandmaster": "RANKED_SOLO_5x5",
    "master": "RANKED_SOLO_5x5",
}


def load_wingbot_riot_config() -> dict:
    """Read Riot settings from WingBot app config if present."""
    if os.name == "nt":
        base = Path(os.environ.get("APPDATA", "")) / "WingBot"
    else:
        base = Path.home() / ".config" / "wingbot"
    path = base / "wingbot_config.json"
    if not path.exists():
        return {}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
        return data.get("riot_api") or {}
    except (json.JSONDecodeError, OSError):
        return {}


class RiotClient:
    def __init__(
        self,
        api_key: str,
        *,
        platform: str = "euw1",
        routing: str | None = None,
        min_interval_sec: float = 1.25,
    ) -> None:
        key = api_key.strip().strip('"').strip("'")
        if not key.startswith("RGAPI-"):
            raise ValueError("Riot API key must start with RGAPI-")
        self.api_key = key
        self.platform = platform.lower()
        self.routing = routing or PLATFORM_TO_ROUTING.get(self.platform, "europe")
        self.min_interval_sec = min_interval_sec
        self._last_request = 0.0

    @classmethod
    def from_env_or_config(cls, **kwargs) -> RiotClient:
        cfg = load_wingbot_riot_config()
        api_key = (
            os.environ.get("RIOT_API_KEY")
            or os.environ.get("RGAPI_KEY")
            or cfg.get("api_key")
            or ""
        )
        if not api_key:
            raise ValueError(
                "No Riot API key. Set RIOT_API_KEY env var or add riot_api.api_key in WingBot Settings."
            )
        platform = kwargs.pop("platform", None) or cfg.get("platform") or "euw1"
        routing = kwargs.pop("routing", None) or cfg.get("routing_region") or None
        return cls(api_key, platform=platform, routing=routing, **kwargs)

    def _wait(self) -> None:
        elapsed = time.monotonic() - self._last_request
        if elapsed < self.min_interval_sec:
            time.sleep(self.min_interval_sec - elapsed)

    def _get(self, base: str, path: str) -> dict | list:
        self._wait()
        url = f"{base}{path}"
        req = urllib.request.Request(
            url,
            headers={"X-Riot-Token": self.api_key, "User-Agent": "wingbot-llm/1.0"},
        )
        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                self._last_request = time.monotonic()
                return json.loads(resp.read().decode())
        except urllib.error.HTTPError as e:
            body = e.read().decode(errors="replace")
            if e.code in (401, 403):
                raise RuntimeError(
                    f"Riot API auth failed ({e.code}). Regenerate your dev key at developer.riotgames.com"
                ) from e
            if e.code == 429:
                retry_after = float(e.headers.get("Retry-After", "10"))
                time.sleep(retry_after)
                return self._get(base, path)
            raise RuntimeError(f"Riot API {e.code} for {path}: {body[:200]}") from e

    def platform_url(self) -> str:
        return f"https://{self.platform}.api.riotgames.com"

    def routing_url(self) -> str:
        return f"https://{self.routing}.api.riotgames.com"

    def league_entries(self, tier: str) -> list[dict]:
        tier = tier.lower()
        queue = LEAGUE_QUEUES.get(tier, "RANKED_SOLO_5x5")
        path = f"/lol/league/v4/{tier}leagues/by-queue/{queue}"
        data = self._get(self.platform_url(), path)
        return data.get("entries") or []

    def match_ids_for_puuid(self, puuid: str, *, start: int = 0, count: int = 20) -> list[str]:
        q = urllib.parse.urlencode({"start": start, "count": count, "type": "ranked"})
        path = f"/lol/match/v5/matches/by-puuid/{puuid}/ids?{q}"
        data = self._get(self.routing_url(), path)
        return list(data) if isinstance(data, list) else []

    def match_by_id(self, match_id: str) -> dict:
        path = f"/lol/match/v5/matches/{match_id}"
        data = self._get(self.routing_url(), path)
        if not isinstance(data, dict):
            raise RuntimeError(f"Unexpected match payload for {match_id}")
        return data

    def timeline_by_id(self, match_id: str) -> dict:
        path = f"/lol/match/v5/matches/{match_id}/timeline"
        data = self._get(self.routing_url(), path)
        if not isinstance(data, dict):
            raise RuntimeError(f"Unexpected timeline payload for {match_id}")
        return data
