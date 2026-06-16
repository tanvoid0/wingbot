#!/usr/bin/env python3
"""Evaluate a project's model against held-out eval.jsonl."""

from __future__ import annotations

import argparse
import json
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from project_loader import get_project_dir, load_project, register_model  # noqa: E402

OLLAMA_URL = "http://127.0.0.1:11434/api/chat"


def load_jsonl(path: Path) -> list[dict]:
    rows = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            rows.append(json.loads(line))
    return rows


def query_ollama(model: str, user_content: str, timeout: int = 120) -> str | None:
    payload = json.dumps({
        "model": model,
        "stream": False,
        "messages": [{"role": "user", "content": user_content}],
    }).encode()
    req = urllib.request.Request(
        OLLAMA_URL,
        data=payload,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            data = json.loads(resp.read().decode())
            return data.get("message", {}).get("content", "")
    except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as e:
        print(f"  Ollama unavailable: {e}", file=sys.stderr)
        return None


def score_response(user_content: str, expected: str, actual: str | None) -> dict:
    if actual is None:
        return {"pass": False, "reason": "no_response", "score": 0.0}

    word_count = len(actual.split())
    too_long = word_count > 100
    too_short = word_count < 5

    # Extract champion names from user JSON
    champs: set[str] = set()
    try:
        state = json.loads(user_content)
        for key in ("my_champ", "enemy_mid", "enemy_top", "enemy_jungle", "enemy_adc", "enemy_support"):
            if state.get(key):
                champs.add(state[key].lower())
        for lst in ("enemies", "enemy_threats", "allies"):
            for c in state.get(lst) or []:
                champs.add(c.lower())
    except json.JSONDecodeError:
        pass

    # Check if response mentions relevant entities or expected keywords
    actual_lower = actual.lower()
    expected_tokens = set(re.findall(r"[A-Za-z']{4,}", expected.lower()))
    overlap = sum(1 for t in expected_tokens if t in actual_lower)
    keyword_score = min(1.0, overlap / max(1, len(expected_tokens) * 0.3))

    champ_mentions = sum(1 for c in champs if c in actual_lower)
    champ_score = min(1.0, champ_mentions / max(1, len(champs) * 0.5)) if champs else 0.5

    length_score = 0.0 if too_long or too_short else 1.0
    total = (keyword_score * 0.5 + champ_score * 0.3 + length_score * 0.2)

    passed = total >= 0.4 and not too_long
    return {
        "pass": passed,
        "score": round(total, 3),
        "word_count": word_count,
        "keyword_score": round(keyword_score, 3),
        "champ_score": round(champ_score, 3),
        "reason": "too_long" if too_long else ("too_short" if too_short else "ok"),
    }


def eval_offline(examples: list[dict]) -> dict:
    """Validate dataset structure without calling Ollama."""
    results = []
    for ex in examples:
        user = ex["messages"][0]["content"]
        expected = ex["messages"][1]["content"]
        try:
            json.loads(user)
            valid_json = True
        except json.JSONDecodeError:
            valid_json = False
        word_ok = 5 <= len(expected.split()) <= 100
        results.append({
            "pass": valid_json and word_ok,
            "score": 1.0 if (valid_json and word_ok) else 0.0,
            "valid_json": valid_json,
        })
    passed = sum(1 for r in results if r["pass"])
    return {
        "mode": "offline",
        "total": len(results),
        "passed": passed,
        "pass_rate": round(passed / max(1, len(results)), 3),
        "results": results,
    }


def eval_online(examples: list[dict], model: str) -> dict:
    results = []
    for i, ex in enumerate(examples):
        user = ex["messages"][0]["content"]
        expected = ex["messages"][1]["content"]
        print(f"  [{i + 1}/{len(examples)}] querying {model}...")
        actual = query_ollama(model, user)
        r = score_response(user, expected, actual)
        r["expected"] = expected[:80]
        r["actual"] = (actual or "")[:120]
        results.append(r)

    passed = sum(1 for r in results if r["pass"])
    avg_score = sum(r["score"] for r in results) / max(1, len(results))
    return {
        "mode": "online",
        "model": model,
        "total": len(results),
        "passed": passed,
        "pass_rate": round(passed / max(1, len(results)), 3),
        "avg_score": round(avg_score, 3),
        "results": results,
    }


def run_eval(project: str, offline: bool = False, model: str | None = None) -> dict:
    project_dir = get_project_dir(project)
    manifest = load_project(project)
    eval_path = project_dir / "datasets" / "eval.jsonl"
    if not eval_path.exists():
        raise FileNotFoundError(f"Missing {eval_path}. Run build_dataset.py first.")

    examples = load_jsonl(eval_path)
    if offline:
        report = eval_offline(examples)
    else:
        model = model or manifest.get("ollama_tag", project)
        report = eval_online(examples, model)

    report_path = project_dir / "datasets" / "eval_report.json"
    report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")

    # Update registry eval_score if online
    if not offline and report.get("pass_rate") is not None:
        from project_loader import load_registry, save_registry

        registry = load_registry()
        tag = model or manifest.get("ollama_tag", project)
        for m in registry.get("models", []):
            if m.get("ollama_tag") == tag:
                m["eval_score"] = report.get("avg_score", report.get("pass_rate"))
        save_registry(registry)

    print(f"Eval: {report['passed']}/{report['total']} passed ({report['pass_rate']:.0%})")
    print(f"Report: {report_path}")
    return report


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--project", required=True)
    parser.add_argument("--offline", action="store_true", help="Validate eval set without Ollama")
    parser.add_argument("--model", help="Ollama model tag (default: project ollama_tag)")
    args = parser.parse_args()
    try:
        run_eval(args.project, offline=args.offline, model=args.model)
        return 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
