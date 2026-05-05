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
client.py — multi-language SDK test runner for RobustMQ chaos testing.

Runs pre-written scenario scripts under:
  ~/.hermes/skills/robustmq-chaos-test/sdk_clients/<sdk>/<scenario>.sh

Each script:
  - Receives CLUSTER_ENDPOINT as an environment variable.
  - Must print a JSON object as its last stdout line:
      {"sent": N, "received": N, "lost": N, "p99_ms": N, "errors": []}
  - Exit code 0 = script ran correctly (pass/fail from JSON fields).
  - Non-zero exit code = script itself failed.

Version management (per-sdk):
  python  → pyenv shell <version>
  go      → gvm use <version>
  rust    → rustup override set <version>
  java    → sdk use java <version>   (sdkman)

When sdk is omitted all four languages run concurrently via
ThreadPoolExecutor — concurrency lives here in Python, not in
Hermes delegate.

Special status: script_format_error
  Last stdout line was not valid protocol JSON. Reported separately from
  test failure so callers can distinguish "RobustMQ broke something" from
  "the test script itself is malformed."
"""

import json
import logging
import os
import subprocess
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Optional

from tools.registry import registry

logger = logging.getLogger(__name__)

SUPPORTED_SDKS = ("python", "go", "rust", "java")
_EXPECTED_KEYS = frozenset({"sent", "received", "lost", "p99_ms", "errors"})

_SDK_CLIENTS_DIR = (
    Path.home() / ".hermes" / "skills" / "robustmq-chaos-test" / "sdk_clients"
)

# Version-manager shell snippets: each must export the right toolchain
# so the subsequent `bash <script>` inherits it.
# We prepend these to the script invocation via `bash -c "..."`.
_VERSION_SETUP: dict = {
    "python": 'eval "$(pyenv init -)" && pyenv shell {version} && ',
    "go":     'source "$HOME/.gvm/scripts/gvm" && gvm use {version} && ',
    "rust":   'source "$HOME/.cargo/env" && rustup override set {version} && ',
    "java":   'source "$HOME/.sdkman/bin/sdkman-init.sh" && sdk use java {version} && ',
}


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _parse_last_json_line(stdout: str) -> Optional[dict]:
    """Return dict if last non-empty stdout line is valid protocol JSON, else None."""
    for line in reversed(stdout.strip().splitlines()):
        line = line.strip()
        if not line:
            continue
        try:
            data = json.loads(line)
            if isinstance(data, dict) and _EXPECTED_KEYS.issubset(data.keys()):
                return data
        except json.JSONDecodeError:
            pass
        # Found a non-empty line but it's not valid protocol JSON — stop here.
        return None
    return None


def _run_one(sdk: str, version: str, scenario: str,
             cluster_endpoint: str, timeout_seconds: int) -> dict:
    script = _SDK_CLIENTS_DIR / sdk / f"{scenario}.sh"
    if not script.exists():
        return {
            "sdk": sdk, "version": version, "scenario": scenario,
            "status": "script_not_found", "passed": False,
            "error": f"script not found: {script}",
        }

    version_prefix = _VERSION_SETUP.get(sdk, "").format(version=version)
    cmd = f'{version_prefix}bash "{script}"'

    env = {**os.environ, "CLUSTER_ENDPOINT": cluster_endpoint}

    t0 = time.monotonic()
    try:
        result = subprocess.run(
            cmd,
            shell=True,
            capture_output=True,
            text=True,
            timeout=timeout_seconds,
            env=env,
        )
    except subprocess.TimeoutExpired:
        return {
            "sdk": sdk, "version": version, "scenario": scenario,
            "status": "timeout", "passed": False,
            "duration_seconds": round(time.monotonic() - t0, 1),
        }

    duration = round(time.monotonic() - t0, 1)

    protocol_data = _parse_last_json_line(result.stdout)
    if protocol_data is None:
        return {
            "sdk": sdk, "version": version, "scenario": scenario,
            "status": "script_format_error", "passed": False,
            "exit_code": result.returncode,
            "duration_seconds": duration,
            "error": (
                "Last stdout line is not valid protocol JSON. "
                "Script must end with: "
                '{"sent": N, "received": N, "lost": N, "p99_ms": N, "errors": []}'
            ),
            "stdout_tail": result.stdout[-300:],
        }

    sent = int(protocol_data.get("sent", 0))
    received = int(protocol_data.get("received", 0))
    lost = int(protocol_data.get("lost", 0))
    p99_ms = protocol_data.get("p99_ms")
    errors = protocol_data.get("errors", [])
    passed = result.returncode == 0

    return {
        "sdk": sdk, "version": version, "scenario": scenario,
        "exit_code": result.returncode,
        "passed": passed,
        "status": "passed" if passed else "failed",
        "sent": sent,
        "received": received,
        "lost": lost,
        "loss_rate": round(lost / sent, 6) if sent > 0 else 0.0,
        "p99_ms": float(p99_ms) if p99_ms is not None else None,
        "errors": errors if isinstance(errors, list) else [str(errors)],
        "duration_seconds": duration,
    }


def _run_all(version: str, scenario: str,
             cluster_endpoint: str, timeout_seconds: int) -> dict:
    results: dict = {}
    with ThreadPoolExecutor(max_workers=len(SUPPORTED_SDKS)) as pool:
        futures = {
            pool.submit(
                _run_one, sdk, version, scenario, cluster_endpoint, timeout_seconds
            ): sdk
            for sdk in SUPPORTED_SDKS
        }
        for future in as_completed(futures):
            sdk = futures[future]
            try:
                results[sdk] = future.result()
            except Exception as exc:
                logger.exception("client: unexpected error for sdk '%s'", sdk)
                results[sdk] = {
                    "sdk": sdk, "scenario": scenario,
                    "status": "runner_error", "passed": False,
                    "error": str(exc),
                }

    failed_sdks = [s for s, r in results.items() if not r.get("passed")]
    return {
        "results": results,
        "all_passed": len(failed_sdks) == 0,
        "failed_sdks": failed_sdks,
    }


# ---------------------------------------------------------------------------
# Tool handler
# ---------------------------------------------------------------------------

def _client_handler(args: dict, **_) -> str:
    action = args.get("action", "")
    if action != "run":
        return json.dumps(
            {"error": f"unknown action: '{action}'. Valid: run"}
        )

    sdk: Optional[str] = args.get("sdk")
    version: str = args.get("version") or "default"
    scenario: str = args.get("scenario") or ""
    cluster_endpoint: str = args.get("cluster_endpoint") or ""
    timeout_seconds: int = int(args.get("timeout_seconds") or 300)

    if not scenario:
        return json.dumps({"error": "scenario is required"})
    if not cluster_endpoint:
        return json.dumps({"error": "cluster_endpoint is required"})

    try:
        if sdk:
            if sdk not in SUPPORTED_SDKS:
                return json.dumps(
                    {"error": f"unknown sdk: '{sdk}'. Valid: {', '.join(SUPPORTED_SDKS)}"}
                )
            result = _run_one(sdk, version, scenario, cluster_endpoint, timeout_seconds)
        else:
            result = _run_all(version, scenario, cluster_endpoint, timeout_seconds)
    except Exception as exc:
        logger.exception("client: unhandled error")
        result = {"error": f"internal error: {exc}"}

    return json.dumps(result, ensure_ascii=False)


# ---------------------------------------------------------------------------
# Schema + registration
# ---------------------------------------------------------------------------

_SCHEMA: dict = {
    "name": "client",
    "description": (
        "Run SDK scenario tests against the RobustMQ cluster. "
        "Scripts live at sdk_clients/<sdk>/<scenario>.sh and receive "
        "CLUSTER_ENDPOINT as an env var. "
        "stdout last line must be JSON: "
        "{sent, received, lost, p99_ms, errors}. "
        "status=script_format_error means the script output was malformed "
        "(distinct from a test failure). "
        "When sdk is omitted, all languages run concurrently."
    ),
    "parameters": {
        "type": "object",
        "properties": {
            "action": {
                "type": "string",
                "enum": ["run"],
                "description": "run: execute the scenario script(s).",
            },
            "sdk": {
                "type": "string",
                "enum": list(SUPPORTED_SDKS),
                "description": "Language SDK. Omit to run all concurrently.",
            },
            "version": {
                "type": "string",
                "description": (
                    "Toolchain version to select via the version manager "
                    "(e.g. '3.11' for Python, '1.21' for Go, 'stable' for Rust, "
                    "'21.0.1-open' for Java). "
                    "Defaults to 'default' which skips the version switch."
                ),
            },
            "scenario": {
                "type": "string",
                "description": (
                    "Scenario name matching the script filename without .sh, "
                    "e.g. 'basic-pubsub', 'failover', 'latency'."
                ),
            },
            "cluster_endpoint": {
                "type": "string",
                "description": (
                    "Cluster address passed as CLUSTER_ENDPOINT. "
                    "Use the value returned by cluster_manage(start)."
                ),
            },
            "timeout_seconds": {
                "type": "integer",
                "description": "Per-script timeout in seconds. Default: 300.",
                "default": 300,
            },
        },
        "required": ["action", "scenario", "cluster_endpoint"],
    },
}

registry.register(
    name="client",
    toolset="chaos",
    schema=_SCHEMA,
    handler=_client_handler,
    emoji="🧪",
)
