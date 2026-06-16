#!/usr/bin/env python3
"""Append a single human-reviewed example to approved gameplay data."""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from project_loader import get_project_dir  # noqa: E402


def main() -> int:
    parser = argparse.ArgumentParser(description="Add one approved training example")
    parser.add_argument("--project", required=True)
    parser.add_argument("--input-json", required=True, help="Compact state JSON string")
    parser.add_argument("--response", required=True, help="Assistant response text")
    parser.add_argument("--source", default="manual")
    args = parser.parse_args()

    project_dir = get_project_dir(args.project)
    approved_path = project_dir / "datasets" / "gameplay" / "approved.jsonl"
    approved_path.parent.mkdir(parents=True, exist_ok=True)

    try:
        inp = json.loads(args.input_json)
    except json.JSONDecodeError as e:
        print(f"Invalid input JSON: {e}", file=sys.stderr)
        return 1

    row = {
        "messages": [
            {"role": "user", "content": json.dumps(inp, separators=(",", ":"))},
            {"role": "assistant", "content": args.response},
        ],
        "source": args.source,
        "captured_at": datetime.now(timezone.utc).isoformat(),
    }

    with approved_path.open("a", encoding="utf-8") as f:
        f.write(json.dumps(row, ensure_ascii=False) + "\n")

    print(f"Appended to {approved_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
