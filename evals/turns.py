#!/usr/bin/env python3
"""Where an agent's turns go, from the sessions run.py saves.

    python3 evals/turns.py evals/results/0.22.0-20261023T190344Z/

For every session: tool calls by kind, how many times it ran `oxframe
check`, how many of those runs did not compile (and with which rustc
errors), what it read (the skill and its reference, the SDK's sources in
the cargo registry, storyboards and crops), and what the eval's sandbox
refused. Then the same, summed over the run, and the compile errors seen
most often: the API the agents trip over.
"""

from __future__ import annotations

import collections
import json
import re
import sys
from pathlib import Path

ERROR_CODE = re.compile(r"error\[(E\d{4})\]: ([^\n|]+)")
CHECK_FAILED = re.compile(r"could not compile|error\[E\d{4}\]|^error: ", re.M)


def text_of(content) -> str:
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        return "\n".join(c.get("text", "") for c in content if isinstance(c, dict))
    return ""


def kind_of_bash(cmd: str) -> str:
    c = cmd.strip()
    for word, kind in [
        ("oxframe check", "check"),
        ("oxframe render", "render"),
        ("oxframe snapshot", "snapshot"),
        ("oxframe inspect", "inspect"),
        ("oxframe add", "add"),
        ("oxframe ", "oxframe other"),
        ("cargo run", "cargo run"),
        ("cargo check", "cargo check"),
        ("cargo build", "cargo build"),
    ]:
        if word in c:
            return kind
    return "shell"


def kind_of_path(path: str) -> str:
    if "/.cargo/registry/" in path or "/oxframe-sdk/" in path:
        return "sdk source"
    if "/.claude/skills/" in path or path.endswith(("AGENTS.md", "CLAUDE.md")):
        return "skill/agents"
    if "/out/check/" in path or "/out/snapshots/" in path:
        return "frames"
    if "/src/" in path:
        return "project src"
    return "other"


def session(path: Path) -> dict:
    tools = collections.Counter()
    reads = collections.Counter()
    errors = collections.Counter()
    pending: dict[str, str] = {}
    refused: set[str] = set()
    checks = failed = denied = 0
    turns = cost = None
    for line in path.read_text().splitlines():
        if not line.strip():
            continue
        e = json.loads(line)
        t = e.get("type")
        if t == "assistant":
            for b in e["message"]["content"]:
                if b.get("type") != "tool_use":
                    continue
                name, inp = b["name"], b.get("input", {})
                if name == "Bash":
                    kind = kind_of_bash(inp.get("command", ""))
                    tools[f"bash: {kind}"] += 1
                    pending[b["id"]] = kind
                elif name in ("Read", "Grep", "Glob"):
                    tools[name] += 1
                    reads[kind_of_path(inp.get("file_path") or inp.get("path") or "")] += 1
                else:
                    tools[name] += 1
        elif t == "user" and isinstance(e["message"].get("content"), list):
            for b in e["message"]["content"]:
                if b.get("type") != "tool_result":
                    continue
                kind = pending.pop(b.get("tool_use_id"), None)
                if b.get("tool_use_id") in refused:
                    continue
                body = text_of(b.get("content"))
                if kind in ("check", "cargo check", "cargo build", "cargo run", "render"):
                    if kind == "check":
                        checks += 1
                    if CHECK_FAILED.search(body):
                        failed += 1
                        for code, msg in ERROR_CODE.findall(body):
                            errors[f"{code}: {msg.strip()[:90]}"] += 1
        elif t == "system" and e.get("subtype") == "permission_denied":
            denied += 1
            refused.add(e.get("tool_use_id"))
        elif t == "result":
            turns, cost = e.get("num_turns"), e.get("total_cost_usd")
    return {
        "id": path.stem,
        "turns": turns,
        "cost": cost,
        "tools": tools,
        "reads": reads,
        "checks": checks,
        "builds_failed": failed,
        "denied": denied,
        "errors": errors,
    }


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    sessions = [session(p) for p in sorted(Path(sys.argv[1]).glob("*.jsonl"))]
    if not sessions:
        print(f"no sessions in {sys.argv[1]}")
        return 1
    print(f"{'prompt':<26}{'turns':>6}{'checks':>8}{'no build':>10}{'sdk reads':>11}{'denied':>8}")
    for s in sessions:
        print(
            f"{s['id']:<26}{s['turns'] or 0:>6}{s['checks']:>8}{s['builds_failed']:>10}"
            f"{s['reads']['sdk source']:>11}{s['denied']:>8}"
        )
    n = len(sessions)
    tools = sum((s["tools"] for s in sessions), collections.Counter())
    reads = sum((s["reads"] for s in sessions), collections.Counter())
    errors = sum((s["errors"] for s in sessions), collections.Counter())
    turns = [s["turns"] for s in sessions if s["turns"]]
    print(f"\n{n} sessions, mean turns {sum(turns) / max(1, len(turns)):.1f}")
    print("tool calls per session:")
    for k, v in tools.most_common():
        print(f"  {k:<22}{v / n:>6.1f}")
    print("reads per session:")
    for k, v in reads.most_common():
        print(f"  {k:<22}{v / n:>6.1f}")
    print(
        f"checks per session {sum(s['checks'] for s in sessions) / n:.1f}, "
        f"of which did not build {sum(s['builds_failed'] for s in sessions) / n:.1f}; "
        f"refused by the eval sandbox {sum(s['denied'] for s in sessions) / n:.1f}"
    )
    print("most frequent compile errors:")
    for k, v in errors.most_common(15):
        print(f"  {v:>3}  {k}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
