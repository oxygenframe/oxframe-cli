#!/usr/bin/env python3
"""Reference prompts for AI agents (ROADMAP A7).

Each prompt in evals/prompts/*.toml is given to a coding agent in a fresh
project from `oxframe init`. Afterwards the runner measures what a user
would care about: did it build, does `oxframe check` pass, did it do what
was asked, and what did it cost (turns, time, tokens, dollars).

    python3 evals/run.py                      # every prompt, Claude Code
    python3 evals/run.py --only camera_follow,chart_pie
    python3 evals/run.py --agent none         # no agent: checks the pipeline
    python3 evals/run.py --agent 'cmd:my-agent --prompt-file {prompt_file}'

Results go to evals/results/<oxframe version>-<UTC time>.json and a table
is printed; with a previous result for the same agent, regressions are
flagged. Standard library only.
"""

from __future__ import annotations

import argparse
import concurrent.futures as cf
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
PROMPTS = HERE / "prompts"
RESULTS = HERE / "results"

# Features that are not in the SDK yet; prompts that need them are skipped.
AVAILABLE = {"camera", "components", "themes", "aspects", "transitions", "sound"}

ASPECT_HINTS = {
    "16:9": r"Landscape|16\s*:\s*9",
    "9:16": r"Portrait|9\s*:\s*16",
    "1:1": r"Square|1\s*:\s*1",
}


@dataclass
class Prompt:
    id: str
    category: str
    prompt: str
    requires: str | None = None
    expect: dict = field(default_factory=dict)


@dataclass
class Result:
    id: str
    category: str
    skipped: str | None = None
    built: bool = False
    check_errors: int | None = None
    check_warnings: int | None = None
    seconds_of_video: float | None = None
    expectations_failed: list[str] = field(default_factory=list)
    passed: bool = False
    turns: int | None = None
    wall_seconds: float = 0.0
    input_tokens: int | None = None
    output_tokens: int | None = None
    cost_usd: float | None = None
    agent_error: str | None = None
    project: str | None = None


def load_prompts(only: set[str] | None) -> list[Prompt]:
    out = []
    for path in sorted(PROMPTS.glob("*.toml")):
        if only and path.stem not in only:
            continue
        data = tomllib.loads(path.read_text())
        out.append(
            Prompt(
                id=path.stem,
                category=data["category"],
                prompt=data["prompt"],
                requires=data.get("requires"),
                expect=data.get("expect", {}),
            )
        )
    if only and len(out) != len(only):
        missing = only - {p.id for p in out}
        sys.exit(f"unknown prompt(s): {', '.join(sorted(missing))}")
    return out


def sh(cmd: list[str], cwd: Path, timeout: float, env: dict | None = None):
    return subprocess.run(
        cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout, env=env
    )


def run_agent(agent: str, prompt: Prompt, project: Path, args) -> dict:
    """Runs the agent in `project`; returns turns/tokens/cost when it says."""
    if agent == "none":
        return {}
    if agent == "claude":
        cmd = [
            "claude", "-p", prompt.prompt,
            "--output-format", "json",
            "--permission-mode", "acceptEdits",
            "--allowedTools", "Bash(oxframe:*) Bash(cargo:*) Read Edit Write Glob Grep",
            "--max-budget-usd", str(args.budget),
        ]
        if args.model:
            cmd += ["--model", args.model]
    elif agent.startswith("cmd:"):
        prompt_file = project / ".eval-prompt.txt"
        prompt_file.write_text(prompt.prompt)
        cmd = shlex.split(agent[4:].format(prompt_file=prompt_file, prompt=prompt.prompt))
    else:
        sys.exit(f"unknown agent {agent!r}: claude, none or cmd:<command>")
    r = sh(cmd, project, args.timeout)
    if r.returncode != 0:
        raise RuntimeError((r.stderr or r.stdout).strip()[-800:])
    if agent != "claude":
        return {}
    out = json.loads(r.stdout)
    usage = out.get("usage", {})
    return {
        "turns": out.get("num_turns"),
        "input_tokens": usage.get("input_tokens", 0)
        + usage.get("cache_read_input_tokens", 0)
        + usage.get("cache_creation_input_tokens", 0),
        "output_tokens": usage.get("output_tokens"),
        "cost_usd": out.get("total_cost_usd"),
    }


def scene_sources(project: Path) -> str:
    return "\n".join(p.read_text() for p in (project / "src").rglob("*.rs"))


def evaluate(prompt: Prompt, project: Path, args, res: Result) -> None:
    env = dict(os.environ, CARGO_TERM_COLOR="never")
    check = sh([args.oxframe, "--json", "check"], project, args.timeout, env)
    try:
        reply = json.loads(check.stdout)
    except json.JSONDecodeError:
        reply = {}
    report_path = project / "out/check/report.json"
    report = json.loads(report_path.read_text()) if report_path.exists() else {}
    res.built = report_path.exists()
    issues = report.get("issues", [])
    if res.built:
        res.check_errors = sum(i.get("severity") == "error" for i in issues)
        res.check_warnings = sum(i.get("severity") == "warning" for i in issues)
        res.seconds_of_video = report.get("duration")
    elif reply.get("stderr"):
        res.expectations_failed.append("does not build")

    src = scene_sources(project)
    exp = prompt.expect
    for name in exp.get("uses", []):
        if not re.search(rf"\b{re.escape(name)}\b", src):
            res.expectations_failed.append(f"does not use {name}")
    for aspect in exp.get("aspects", []):
        if not re.search(ASPECT_HINTS.get(aspect, re.escape(aspect)), src):
            res.expectations_failed.append(f"no {aspect} format")
    if "min_seconds" in exp and (res.seconds_of_video or 0) < exp["min_seconds"]:
        res.expectations_failed.append(
            f"{res.seconds_of_video or 0:.1f} s of video, wanted ≥ {exp['min_seconds']}"
        )
    res.passed = res.built and res.check_errors == 0 and not res.expectations_failed


def run_one(prompt: Prompt, args, workdir: Path) -> Result:
    res = Result(id=prompt.id, category=prompt.category)
    if prompt.requires and prompt.requires not in AVAILABLE:
        res.skipped = f"needs {prompt.requires}"
        return res
    project = workdir / prompt.id
    init = [args.oxframe, "init", prompt.id]
    if args.sdk_path:
        init += ["--sdk-path", args.sdk_path]
    r = sh(init, workdir, 300)
    if r.returncode != 0:
        res.agent_error = f"oxframe init failed: {r.stderr.strip()[-400:]}"
        return res
    res.project = str(project)
    started = time.monotonic()
    try:
        for k, v in run_agent(args.agent, prompt, project, args).items():
            setattr(res, k, v)
    except subprocess.TimeoutExpired:
        res.agent_error = f"agent timed out after {args.timeout:.0f} s"
    except Exception as e:  # the agent failed; still measure the project
        res.agent_error = str(e)
    res.wall_seconds = round(time.monotonic() - started, 1)
    evaluate(prompt, project, args, res)
    return res


def oxframe_version(oxframe: str) -> str:
    out = subprocess.run([oxframe, "--version"], capture_output=True, text=True).stdout
    return out.split()[-1] if out.strip() else "unknown"


def summary(results: list[Result]) -> dict:
    ran = [r for r in results if not r.skipped]
    n = len(ran) or 1

    def mean(xs):
        xs = [x for x in xs if x is not None]
        return round(sum(xs) / len(xs), 2) if xs else None

    return {
        "prompts": len(ran),
        "skipped": len(results) - len(ran),
        "pass_rate": round(sum(r.passed for r in ran) / n, 3),
        "build_rate": round(sum(r.built for r in ran) / n, 3),
        "mean_turns": mean(r.turns for r in ran),
        "mean_minutes": mean(r.wall_seconds / 60 for r in ran),
        "mean_tokens": mean(
            (r.input_tokens or 0) + (r.output_tokens or 0) if r.input_tokens is not None else None
            for r in ran
        ),
        "total_cost_usd": round(sum(r.cost_usd or 0 for r in ran), 2),
    }


def previous(agent: str, current: Path) -> dict | None:
    runs = sorted(p for p in RESULTS.glob("*.json") if p != current)
    for p in reversed(runs):
        data = json.loads(p.read_text())
        if data.get("agent") == agent:
            return data
    return None


def regressions(now: dict, before: dict) -> list[str]:
    out = []
    a, b = now["summary"], before["summary"]
    if a["pass_rate"] < b["pass_rate"] - 0.1:
        out.append(f"pass rate {b['pass_rate']:.0%} → {a['pass_rate']:.0%}")
    if a["mean_turns"] and b["mean_turns"] and a["mean_turns"] > b["mean_turns"] * 1.25:
        out.append(f"mean turns {b['mean_turns']} → {a['mean_turns']}")
    if a["mean_tokens"] and b["mean_tokens"] and a["mean_tokens"] > b["mean_tokens"] * 1.25:
        out.append(f"mean tokens {b['mean_tokens']:.0f} → {a['mean_tokens']:.0f}")
    was = {r["id"]: r["passed"] for r in before["results"]}
    for r in now["results"]:
        if was.get(r["id"]) and not r["passed"] and not r["skipped"]:
            out.append(f"{r['id']} passed before, fails now")
    return out


def table(results: list[Result]) -> str:
    rows = [("prompt", "ok", "build", "check err", "video s", "turns", "min", "tokens", "$", "notes")]
    for r in results:
        if r.skipped:
            rows.append((r.id, "–", "", "", "", "", "", "", "", r.skipped))
            continue
        tokens = (r.input_tokens or 0) + (r.output_tokens or 0) if r.input_tokens is not None else None
        notes = "; ".join(filter(None, [r.agent_error and r.agent_error[:60], *r.expectations_failed]))
        rows.append((
            r.id, "✓" if r.passed else "✗", "yes" if r.built else "no",
            "" if r.check_errors is None else str(r.check_errors),
            "" if r.seconds_of_video is None else f"{r.seconds_of_video:.1f}",
            "" if r.turns is None else str(r.turns),
            f"{r.wall_seconds / 60:.1f}",
            "" if tokens is None else f"{tokens / 1000:.0f}k",
            "" if r.cost_usd is None else f"{r.cost_usd:.2f}",
            notes,
        ))
    widths = [max(len(row[i]) for row in rows) for i in range(len(rows[0]))]
    return "\n".join("  ".join(c.ljust(w) for c, w in zip(row, widths)).rstrip() for row in rows)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--agent", default="claude", help="claude (default), none, or cmd:<command with {prompt_file}>")
    ap.add_argument("--model", help="model for the claude agent")
    ap.add_argument("--only", help="comma-separated prompt ids")
    ap.add_argument("--jobs", type=int, default=2, help="prompts run in parallel (default 2)")
    ap.add_argument("--timeout", type=float, default=1200, help="seconds per prompt (default 1200)")
    ap.add_argument("--budget", type=float, default=2.0, help="max dollars per prompt for claude (default 2)")
    ap.add_argument("--sdk-path", help="build the projects against a local oxframe-sdk checkout")
    ap.add_argument("--oxframe", default=shutil.which("oxframe") or "oxframe", help="the oxframe CLI to test")
    ap.add_argument("--keep", action="store_true", help="keep the generated projects")
    args = ap.parse_args()

    prompts = load_prompts(set(args.only.split(",")) if args.only else None)
    version = oxframe_version(args.oxframe)
    workdir = Path(tempfile.mkdtemp(prefix="oxframe-evals-"))
    # One shared target directory: dependencies compile once for all prompts.
    os.environ.setdefault("CARGO_TARGET_DIR", str(workdir / "target"))
    print(f"oxframe {version}, agent {args.agent}, {len(prompts)} prompt(s), projects in {workdir}")

    results: list[Result] = []
    with cf.ThreadPoolExecutor(max_workers=max(1, args.jobs)) as pool:
        futures = {pool.submit(run_one, p, args, workdir): p for p in prompts}
        for fut in cf.as_completed(futures):
            r = fut.result()
            results.append(r)
            state = r.skipped or ("pass" if r.passed else "fail")
            print(f"  {r.id}: {state}", flush=True)
    results.sort(key=lambda r: r.id)

    RESULTS.mkdir(exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    path = RESULTS / f"{version}-{stamp}.json"
    data = {
        "oxframe": version,
        "agent": args.agent,
        "model": args.model,
        "date": stamp,
        "summary": summary(results),
        "results": [asdict(r) for r in results],
    }
    path.write_text(json.dumps(data, indent=2) + "\n")
    print("\n" + table(results))
    s = data["summary"]
    print(
        f"\npass {s['pass_rate']:.0%} ({s['prompts']} run, {s['skipped']} skipped), "
        f"builds {s['build_rate']:.0%}, turns {s['mean_turns']}, "
        f"{s['mean_minutes']} min, ${s['total_cost_usd']} → {path.relative_to(HERE.parent)}"
    )
    before = previous(args.agent, path)
    worse = regressions(data, before) if before else []
    for w in worse:
        print(f"regression vs {before['oxframe']} ({before['date']}): {w}")
    if not args.keep:
        shutil.rmtree(workdir, ignore_errors=True)
    return 1 if worse else 0


if __name__ == "__main__":
    sys.exit(main())
