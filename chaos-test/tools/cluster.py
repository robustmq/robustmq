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
cluster.py — RobustMQ test cluster lifecycle tool.

Spawns 3 broker processes directly on the local host (no Docker).
Each broker gets its own port and a tempdir for data.

Ports:
  broker-1: 1883
  broker-2: 2883
  broker-3: 3883

State is kept in a module-level dict so start/stop/status share it
within the same Hermes session. For cross-session persistence the
caller (Skill) should use chaos_state.

Required env var:
  ROBUSTMQ_HOME  — directory that contains the RobustMQ binary.
                   Fail-fast if unset; no default.
"""

import json
import logging
import os
import shutil
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Optional

from tools.registry import registry

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Module-level state (lives for the lifetime of the Hermes process)
# ---------------------------------------------------------------------------

_BROKERS: dict = {}
# Shape:
# {
#   "broker-1": {
#     "process": subprocess.Popen,
#     "port": 1883,
#     "data_dir": "/tmp/rmq-abc123",
#     "node_name": "broker-1",
#   },
#   ...
# }

_BROKER_PORTS = {
    "broker-1": 1883,
    "broker-2": 2883,
    "broker-3": 3883,
}

_HEALTH_TIMEOUT = 5   # seconds to wait before health check
_HEALTH_URL_TEMPLATE = "http://127.0.0.1:{port}/health"


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _robustmq_binary() -> Optional[str]:
    home = os.environ.get("ROBUSTMQ_HOME", "").strip()
    if not home:
        return None
    # Expect the binary to be at $ROBUSTMQ_HOME/bin/robustmq-server
    # or directly at $ROBUSTMQ_HOME/robustmq-server.
    candidates = [
        Path(home) / "bin" / "robustmq-server",
        Path(home) / "robustmq-server",
    ]
    for c in candidates:
        if c.is_file():
            return str(c)
    # Return the first candidate path so the error message is actionable.
    return str(candidates[0])


def _health_check(port: int) -> bool:
    url = _HEALTH_URL_TEMPLATE.format(port=port)
    try:
        with urllib.request.urlopen(url, timeout=3) as resp:
            return resp.status == 200
    except (urllib.error.URLError, OSError):
        return False


def _kill_all() -> None:
    """Best-effort kill of all tracked broker processes."""
    for name, info in list(_BROKERS.items()):
        proc = info.get("process")
        if proc and proc.poll() is None:
            try:
                proc.kill()
                proc.wait(timeout=5)
            except Exception as exc:
                logger.warning("cluster: failed to kill %s: %s", name, exc)
    _BROKERS.clear()


def _cleanup_data_dirs(data_dirs: list) -> None:
    for d in data_dirs:
        try:
            shutil.rmtree(d, ignore_errors=True)
        except Exception as exc:
            logger.warning("cluster: failed to remove data dir %s: %s", d, exc)


# ---------------------------------------------------------------------------
# Actions
# ---------------------------------------------------------------------------

def _action_start() -> dict:
    binary = _robustmq_binary()
    if binary is None:
        return {
            "error": (
                "ROBUSTMQ_HOME is not set. "
                "Export it to the directory containing the RobustMQ installation, "
                "e.g. export ROBUSTMQ_HOME=/opt/robustmq"
            )
        }
    if not Path(binary).is_file():
        return {
            "error": (
                f"RobustMQ binary not found at {binary}. "
                "Check that ROBUSTMQ_HOME points to a valid installation."
            )
        }

    if _BROKERS:
        return {
            "error": (
                "Cluster is already running. "
                "Call stop first if you want to restart."
            )
        }

    data_dirs: list[str] = []
    started: list[str] = []

    for node_name, port in _BROKER_PORTS.items():
        data_dir = tempfile.mkdtemp(prefix=f"rmq-{node_name}-")
        data_dirs.append(data_dir)

        log_dir = Path(data_dir) / "logs"
        log_dir.mkdir(parents=True, exist_ok=True)
        log_file = log_dir / "broker.log"

        cmd = [
            binary,
            "--node-name", node_name,
            "--mqtt-port", str(port),
            "--data-dir", data_dir,
        ]

        try:
            with open(log_file, "w") as lf:
                proc = subprocess.Popen(
                    cmd,
                    stdout=lf,
                    stderr=subprocess.STDOUT,
                    close_fds=True,
                )
        except OSError as exc:
            _kill_all()
            _cleanup_data_dirs(data_dirs)
            return {"error": f"Failed to start {node_name}: {exc}"}

        _BROKERS[node_name] = {
            "process": proc,
            "port": port,
            "data_dir": data_dir,
            "node_name": node_name,
        }
        started.append(node_name)

    # Wait then health-check the first broker as cluster representative.
    time.sleep(_HEALTH_TIMEOUT)
    if not _health_check(_BROKER_PORTS["broker-1"]):
        _kill_all()
        _cleanup_data_dirs(data_dirs)
        return {
            "status": "failed",
            "error": (
                "Health check failed for broker-1 on port 1883 after "
                f"{_HEALTH_TIMEOUT}s. Check logs in: {data_dirs[0]}/logs/broker.log"
            ),
        }

    return {
        "status": "running",
        "endpoint": "127.0.0.1:1883",
        "nodes": list(_BROKER_PORTS.keys()),
        "ports": _BROKER_PORTS,
        "data_dirs": data_dirs,
    }


def _action_stop() -> dict:
    if not _BROKERS:
        return {"status": "stopped", "note": "No running cluster found."}

    data_dirs = [info["data_dir"] for info in _BROKERS.values()]
    _kill_all()
    _cleanup_data_dirs(data_dirs)
    return {"status": "stopped", "cleaned_dirs": data_dirs}


def _action_status() -> dict:
    if not _BROKERS:
        return {"status": "stopped", "running_processes": 0, "endpoint": None}

    alive = []
    dead = []
    for name, info in _BROKERS.items():
        proc = info.get("process")
        if proc and proc.poll() is None:
            alive.append(name)
        else:
            dead.append(name)

    if not alive:
        status = "stopped"
    elif dead:
        status = "degraded"
    else:
        status = "running"

    return {
        "status": status,
        "running_processes": len(alive),
        "total_processes": len(_BROKERS),
        "alive_nodes": alive,
        "dead_nodes": dead,
        "endpoint": "127.0.0.1:1883" if alive else None,
    }


# ---------------------------------------------------------------------------
# Tool handler
# ---------------------------------------------------------------------------

def _cluster_handler(args: dict, **_) -> str:
    action = args.get("action", "")
    try:
        if action == "start":
            result = _action_start()
        elif action == "stop":
            result = _action_stop()
        elif action == "status":
            result = _action_status()
        else:
            result = {
                "error": f"unknown action: '{action}'. Valid: start, stop, status"
            }
    except Exception as exc:
        logger.exception("cluster: unhandled error in action '%s'", action)
        result = {"error": f"internal error: {exc}"}
    return json.dumps(result, ensure_ascii=False)


# ---------------------------------------------------------------------------
# Schema + registration
# ---------------------------------------------------------------------------

_SCHEMA: dict = {
    "name": "cluster_manage",
    "description": (
        "Start, stop, or query the RobustMQ test cluster. "
        "Spawns 3 broker processes locally (no Docker). "
        "Requires ROBUSTMQ_HOME env var — fails immediately if unset. "
        "start: launches broker-1/2/3 on ports 1883/2883/3883, "
        "waits 5 s, health-checks broker-1, returns endpoint '127.0.0.1:1883'. "
        "stop: kills all brokers and removes their temp data dirs. "
        "status: returns per-node liveness."
    ),
    "parameters": {
        "type": "object",
        "properties": {
            "action": {
                "type": "string",
                "enum": ["start", "stop", "status"],
                "description": (
                    "start: spawn brokers and wait for health. "
                    "stop: kill all brokers and clean up data dirs. "
                    "status: check which broker processes are alive."
                ),
            },
        },
        "required": ["action"],
    },
}

registry.register(
    name="cluster_manage",
    toolset="chaos",
    schema=_SCHEMA,
    handler=_cluster_handler,
    emoji="🖥️",
)
