#!/usr/bin/env python3
"""Merge knowledge packs into _merged/ with a reproducible manifest."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from project_loader import get_project_dir, load_project, project_path  # noqa: E402


def file_hash(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def collect_files(pack_path: Path) -> list[Path]:
    if not pack_path.exists():
        return []
    if pack_path.is_file():
        return [pack_path]
    return sorted(
        p for p in pack_path.rglob("*") if p.is_file() and p.name != "manifest.json"
    )


def merge_packs(project: str, extra_packs: list[str] | None = None) -> Path:
    manifest = load_project(project)
    project_dir = get_project_dir(project)
    packs = list(manifest.get("knowledge_packs", ["knowledge"]))
    if extra_packs:
        packs.extend(extra_packs)

    merged_dir = project_dir / "knowledge" / "_merged"
    merged_dir.mkdir(parents=True, exist_ok=True)

    entries: list[dict] = []
    for pack in packs:
        pack_path = Path(pack)
        if not pack_path.is_absolute():
            pack_path = project_dir / pack
        for src in collect_files(pack_path):
            rel = src.relative_to(project_dir)
            dest = merged_dir / rel
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes(src.read_bytes())
            entries.append({
                "source": str(rel).replace("\\", "/"),
                "sha256": file_hash(src),
                "size_bytes": src.stat().st_size,
            })

    out_manifest = {
        "project": project,
        "merged_at": datetime.now(timezone.utc).isoformat(),
        "packs": packs,
        "files": entries,
        "file_count": len(entries),
    }
    manifest_path = merged_dir / "manifest.json"
    manifest_path.write_text(json.dumps(out_manifest, indent=2), encoding="utf-8")
    print(f"Merged {len(entries)} files -> {manifest_path}")
    return manifest_path


def main() -> int:
    parser = argparse.ArgumentParser(description="Merge knowledge packs for a project")
    parser.add_argument("--project", required=True)
    parser.add_argument("--add", action="append", default=[], help="Extra pack path(s)")
    args = parser.parse_args()
    try:
        merge_packs(args.project, args.add or None)
        return 0
    except FileNotFoundError as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
