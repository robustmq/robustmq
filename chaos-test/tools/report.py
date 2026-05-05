# Copyright 2023 RobustMQ Team
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""
report.py — generate and push chaos test reports for RobustMQ.

Action: generate_and_push

Flow:
  1. Render Markdown via Jinja2 template (templates/report.md.j2).
  2. Write JSON via json.dumps — never calls LLM.
  3. git clone (shallow) / git pull the reports repo, add both files,
     commit, and push using a Deploy Key at ~/.ssh/test-reports-deploy.
  4. Return {json_path, markdown_path, github_url, run_passed}.

Pass/fail logic:
  run_passed = (all core scenarios passed) AND (non-core pass rate >= 75%)
  CORE_SCENARIOS = {"broker-kill-single", "leader-transfer"}

Config keys read from config.yml:
  reports:
    repo_url: "git@github.com:org/test-reports.git"
    branch:   "main"          # optional, default: main

Deploy Key:  ~/.ssh/test-reports-deploy  (must have write access to repo)

Output files are written to a temp dir and cleaned up after push.
Local clones older than 30 days under ~/.hermes/skills/robustmq-chaos-test/report-clones/
are removed on each run.
"""

import json
import logging
import os
import shutil
import subprocess
import tempfile
import time
from datetime import datetime, timezone, timedelta
from pathlib import Path
from typing import Optional

from tools.registry import registry

logger = logging.getLogger(__name__)

_CONFIG_PATH = (
    Path.home() / ".hermes" / "skills" / "robustmq-chaos-test" / "config.yml"
)
_TEMPLATE_PATH = (
    Path.home() / ".hermes" / "skills" / "robustmq-chaos-test"
    / "templates" / "report.md.j2"
)
_CLONE_BASE = (
    Path.home() / ".hermes" / "skills" / "robustmq-chaos-test" / "report-clones"
)
_DEPLOY_KEY = Path.home() / ".ssh" / "test-reports-deploy"

CORE_SCENARIOS = frozenset({"broker-kill-single", "leader-transfer"})


# ---------------------------------------------------------------------------
# Config loader
# ---------------------------------------------------------------------------

def _load_reports_config() -> dict:
    """Read reports.repo_url and reports.branch from config.yml."""
    result = {"repo_url": None, "branch": "main"}
    if not _CONFIG_PATH.exists():
        return result
    try:
        text = _CONFIG_PATH.read_text(encoding="utf-8")
        in_reports = False
        for line in text.splitlines():
            stripped = line.strip()
            if stripped.startswith("reports:"):
                in_reports = True
                continue
            if in_reports:
                if stripped.startswith("repo_url:"):
                    val = stripped.split(":", 1)[1].strip().strip('"').strip("'")
                    if val:
                        result["repo_url"] = val
                elif stripped.startswith("branch:"):
                    val = stripped.split(":", 1)[1].strip().strip('"').strip("'")
                    if val:
                        result["branch"] = val
                elif stripped and not stripped.startswith("#") and ":" in stripped and not line.startswith(" "):
                    in_reports = False
    except OSError as exc:
        logger.error("report: failed to read config.yml: %s", exc)
    return result


# ---------------------------------------------------------------------------
# Pass/fail logic
# ---------------------------------------------------------------------------

def _compute_run_passed(scenarios: list) -> bool:
    """
    run_passed = all core scenarios passed AND non-core pass rate >= 75%.
    A scenario passes if its 'passed' field is True.
    Missing core scenarios count as failed.
    """
    if not scenarios:
        return False

    by_name: dict = {}
    for s in scenarios:
        name = s.get("scenario") or s.get("name") or ""
        by_name[name] = s

    # Core check
    for core in CORE_SCENARIOS:
        s = by_name.get(core)
        if s is None or not s.get("passed"):
            return False

    # Non-core check
    non_core = [s for name, s in by_name.items() if name not in CORE_SCENARIOS]
    if non_core:
        passed_count = sum(1 for s in non_core if s.get("passed"))
        if passed_count / len(non_core) < 0.75:
            return False

    return True


# ---------------------------------------------------------------------------
# Rendering
# ---------------------------------------------------------------------------

def _render_markdown(run_data: dict, run_passed: bool) -> str:
    """Render the Markdown report using Jinja2 template."""
    try:
        from jinja2 import Environment, FileSystemLoader, StrictUndefined
    except ImportError:
        return _render_markdown_fallback(run_data, run_passed)

    template_dir = _TEMPLATE_PATH.parent
    template_name = _TEMPLATE_PATH.name

    if not _TEMPLATE_PATH.exists():
        return _render_markdown_fallback(run_data, run_passed)

    env = Environment(
        loader=FileSystemLoader(str(template_dir)),
        undefined=StrictUndefined,
        autoescape=False,
    )
    template = env.get_template(template_name)
    return template.render(run_data=run_data, run_passed=run_passed,
                           core_scenarios=sorted(CORE_SCENARIOS))


def _render_markdown_fallback(run_data: dict, run_passed: bool) -> str:
    """Minimal Markdown report when Jinja2 or template is unavailable."""
    run_id = run_data.get("run_id", "unknown")
    started_at = run_data.get("started_at", "")
    finished_at = run_data.get("finished_at", "")
    scenarios = run_data.get("scenarios", [])

    status_icon = "✅" if run_passed else "❌"
    status_label = "PASSED" if run_passed else "FAILED"

    lines = [
        f"# RobustMQ Chaos Test Report",
        f"",
        f"**Run ID:** `{run_id}`  ",
        f"**Status:** {status_icon} {status_label}  ",
        f"**Started:** {started_at}  ",
        f"**Finished:** {finished_at}  ",
        f"",
        f"## Scenarios",
        f"",
        f"| Scenario | SDK | Passed | Sent | Lost | p99 ms | Duration s |",
        f"|----------|-----|--------|------|------|--------|------------|",
    ]

    for s in scenarios:
        name = s.get("scenario") or s.get("name") or "-"
        sdk = s.get("sdk") or "all"
        passed = "✅" if s.get("passed") else "❌"
        sent = s.get("sent", "-")
        lost = s.get("lost", "-")
        p99 = s.get("p99_ms", "-")
        dur = s.get("duration_seconds", "-")
        lines.append(f"| {name} | {sdk} | {passed} | {sent} | {lost} | {p99} | {dur} |")

    lines += ["", "---", f"*Generated by robustmq-chaos-test skill*"]
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Git push
# ---------------------------------------------------------------------------

def _git_ssh_env() -> dict:
    return {
        **os.environ,
        "GIT_SSH_COMMAND": f"ssh -i {_DEPLOY_KEY} -o StrictHostKeyChecking=no -o BatchMode=yes",
    }


def _run_git(args: list, cwd: str, env: dict, timeout: int = 60) -> tuple[int, str, str]:
    result = subprocess.run(
        ["git"] + args,
        cwd=cwd,
        capture_output=True,
        text=True,
        timeout=timeout,
        env=env,
    )
    return result.returncode, result.stdout, result.stderr


def _push_to_github(repo_url: str, branch: str,
                    json_path: Path, md_path: Path,
                    run_id: str) -> dict:
    """
    Clone the reports repo into a temp dir, add both report files,
    commit, and push. Returns {"github_url": "...", "commit": "..."} or {"error": "..."}.
    """
    env = _git_ssh_env()
    clone_dir = tempfile.mkdtemp(prefix="rmq-report-push-", dir=str(_CLONE_BASE))

    try:
        # Shallow clone
        rc, _, err = _run_git(
            ["clone", "--depth=1", "--branch", branch, repo_url, clone_dir],
            cwd="/tmp", env=env, timeout=120,
        )
        if rc != 0:
            return {"error": f"git clone failed: {err[:300]}"}

        # Copy report files into clone
        dest_dir = Path(clone_dir) / "reports" / run_id
        dest_dir.mkdir(parents=True, exist_ok=True)

        shutil.copy2(str(json_path), str(dest_dir / json_path.name))
        shutil.copy2(str(md_path), str(dest_dir / md_path.name))

        # git add
        rc, _, err = _run_git(
            ["add", str(dest_dir / json_path.name), str(dest_dir / md_path.name)],
            cwd=clone_dir, env=env,
        )
        if rc != 0:
            return {"error": f"git add failed: {err[:300]}"}

        # git commit
        rc, _, err = _run_git(
            ["commit", "-m", f"chaos: add report for run {run_id}",
             "--author", "robustmq-chaos-bot <chaos@robustmq.bot>"],
            cwd=clone_dir, env=env,
        )
        if rc != 0:
            return {"error": f"git commit failed: {err[:300]}"}

        # Get commit hash
        rc2, sha, _ = _run_git(["rev-parse", "--short", "HEAD"], cwd=clone_dir, env=env)
        sha = sha.strip() if rc2 == 0 else "unknown"

        # git push
        rc, _, err = _run_git(
            ["push", "origin", branch],
            cwd=clone_dir, env=env, timeout=120,
        )
        if rc != 0:
            return {"error": f"git push failed: {err[:300]}"}

        # Build GitHub URL (best-effort: convert SSH to HTTPS URL)
        github_url = _ssh_to_https(repo_url) + f"/tree/{branch}/reports/{run_id}"
        return {"github_url": github_url, "commit": sha}

    except subprocess.TimeoutExpired as exc:
        return {"error": f"git operation timed out: {exc}"}
    except Exception as exc:
        return {"error": f"push error: {exc}"}
    finally:
        try:
            shutil.rmtree(clone_dir, ignore_errors=True)
        except Exception:
            pass


def _ssh_to_https(ssh_url: str) -> str:
    """Convert git@github.com:org/repo.git → https://github.com/org/repo"""
    if ssh_url.startswith("git@"):
        # git@github.com:org/repo.git
        without_prefix = ssh_url[4:]  # github.com:org/repo.git
        host, path = without_prefix.split(":", 1)
        path = path.removesuffix(".git")
        return f"https://{host}/{path}"
    # Already HTTPS or unknown — strip .git suffix
    return ssh_url.removesuffix(".git")


# ---------------------------------------------------------------------------
# Cleanup
# ---------------------------------------------------------------------------

def _cleanup_old_clones(max_age_days: int = 30) -> None:
    if not _CLONE_BASE.exists():
        return
    cutoff = time.time() - max_age_days * 86400
    for entry in _CLONE_BASE.iterdir():
        try:
            if entry.stat().st_mtime < cutoff:
                shutil.rmtree(entry, ignore_errors=True)
        except Exception as exc:
            logger.warning("report: cleanup error for %s: %s", entry, exc)


# ---------------------------------------------------------------------------
# Main action
# ---------------------------------------------------------------------------

def _action_generate_and_push(run_data: dict) -> dict:
    run_id = run_data.get("run_id") or f"run-{int(time.time())}"
    finished_at = run_data.get("finished_at") or datetime.now(timezone.utc).isoformat()

    # Ensure finished_at is recorded
    run_data = {**run_data, "finished_at": finished_at}

    scenarios = run_data.get("scenarios") or []
    run_passed = _compute_run_passed(scenarios)

    # Load config
    cfg = _load_reports_config()
    repo_url = cfg.get("repo_url")
    branch = cfg.get("branch", "main")

    # Render reports into a temp dir
    tmp = tempfile.mkdtemp(prefix=f"rmq-report-{run_id}-")
    try:
        json_path = Path(tmp) / f"{run_id}.json"
        md_path = Path(tmp) / f"{run_id}.md"

        # JSON report
        report_json = {**run_data, "run_passed": run_passed}
        json_path.write_text(
            json.dumps(report_json, ensure_ascii=False, indent=2),
            encoding="utf-8",
        )

        # Markdown report
        md_content = _render_markdown(run_data, run_passed)
        md_path.write_text(md_content, encoding="utf-8")

        result = {
            "json_path": str(json_path),
            "markdown_path": str(md_path),
            "run_passed": run_passed,
            "github_url": None,
        }

        # Push if repo configured and deploy key present
        if not repo_url:
            result["push_skipped"] = (
                "reports.repo_url not set in config.yml — reports generated locally only"
            )
            return result

        if not _DEPLOY_KEY.exists():
            result["push_skipped"] = (
                f"Deploy key not found at {_DEPLOY_KEY} — reports generated locally only"
            )
            return result

        _CLONE_BASE.mkdir(parents=True, exist_ok=True)
        _cleanup_old_clones()

        push_result = _push_to_github(repo_url, branch, json_path, md_path, run_id)
        if "error" in push_result:
            result["push_error"] = push_result["error"]
        else:
            result["github_url"] = push_result.get("github_url")
            result["commit"] = push_result.get("commit")

        return result

    except Exception as exc:
        logger.exception("report: unhandled error generating report for run %s", run_id)
        return {"error": f"internal error: {exc}"}
    # Note: we intentionally do NOT clean up `tmp` here — the caller may
    # need to read the files. Temp dirs are OS-managed.


# ---------------------------------------------------------------------------
# Tool handler
# ---------------------------------------------------------------------------

def _report_handler(args: dict, **_) -> str:
    action = args.get("action", "")
    if action != "generate_and_push":
        return json.dumps(
            {"error": f"unknown action: '{action}'. Valid: generate_and_push"}
        )

    run_data = args.get("run_data")
    if not run_data or not isinstance(run_data, dict):
        return json.dumps({"error": "run_data (dict) is required"})

    try:
        result = _action_generate_and_push(run_data)
    except Exception as exc:
        logger.exception("report: unhandled top-level error")
        result = {"error": f"internal error: {exc}"}

    return json.dumps(result, ensure_ascii=False)


# ---------------------------------------------------------------------------
# Schema + registration
# ---------------------------------------------------------------------------

_SCHEMA: dict = {
    "name": "report",
    "description": (
        "Generate a Markdown + JSON report from a completed chaos test run "
        "and push it to the GitHub reports repo via Deploy Key. "
        "run_passed = all core scenarios pass AND non-core pass rate >= 75%. "
        "Core scenarios: broker-kill-single, leader-transfer. "
        "Returns json_path, markdown_path, github_url (null if push skipped/failed), run_passed. "
        "Push is skipped (not an error) if reports.repo_url is missing from config.yml "
        "or if the deploy key is absent — files are still written locally."
    ),
    "parameters": {
        "type": "object",
        "properties": {
            "action": {
                "type": "string",
                "enum": ["generate_and_push"],
                "description": "generate_and_push: render + commit + push.",
            },
            "run_data": {
                "type": "object",
                "description": (
                    "Full run data dict. Expected keys: "
                    "run_id (str), started_at (ISO), finished_at (ISO, optional — "
                    "defaults to now), scenarios (list of scenario result dicts). "
                    "Each scenario dict should include: scenario, sdk, passed, "
                    "sent, received, lost, p99_ms, duration_seconds, errors."
                ),
            },
        },
        "required": ["action", "run_data"],
    },
}

registry.register(
    name="report",
    toolset="chaos",
    schema=_SCHEMA,
    handler=_report_handler,
    emoji="📊",
)
