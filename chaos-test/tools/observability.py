"""
observability.py — collect logs and metrics from a running RobustMQ cluster.

Actions:
  collect_logs     — tail last N lines from each broker's log file
  collect_metrics  — scrape /metrics (Prometheus) from each broker's HTTP port
  snapshot         — collect_logs + collect_metrics in one call, timestamped

Log files live at:  <data_dir>/logs/broker.log   (written by cluster.py)
Metrics endpoint:   http://127.0.0.1:<http_port>/metrics

HTTP ports (one above each MQTT port):
  broker-1: 1884
  broker-2: 2884
  broker-3: 3884

data_dirs is passed in by the caller (taken from the cluster start return value).
"""

import json
import logging
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

from tools.registry import registry

logger = logging.getLogger(__name__)

# HTTP metrics port = MQTT port + 1
_NODES = {
    "broker-1": {"mqtt_port": 1883, "http_port": 1884},
    "broker-2": {"mqtt_port": 2883, "http_port": 2884},
    "broker-3": {"mqtt_port": 3883, "http_port": 3884},
}

# Prometheus metric names we care about
_METRIC_KEYS = (
    "robustmq_connections_total",
    "robustmq_messages_in_total",
    "robustmq_messages_out_total",
    "robustmq_errors_total",
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _tail_lines(path: Path, n: int) -> list[str]:
    """Return the last n lines of a text file without reading the whole file."""
    if not path.exists():
        return [f"[log file not found: {path}]"]
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            # Simple approach: read all lines and slice.
            # For large log files a seek-from-end approach would be better,
            # but broker logs in short test runs are small.
            lines = f.readlines()
            return [l.rstrip("\n") for l in lines[-n:]]
    except OSError as exc:
        return [f"[error reading log: {exc}]"]


def _parse_prometheus(text: str) -> dict:
    """Extract values for the metric keys we care about from Prometheus text format."""
    result: dict = {}
    for line in text.splitlines():
        line = line.strip()
        if line.startswith("#") or not line:
            continue
        for key in _METRIC_KEYS:
            if line.startswith(key):
                parts = line.split()
                if len(parts) >= 2:
                    short = key.replace("robustmq_", "")
                    try:
                        result[short] = float(parts[-1])
                    except ValueError:
                        result[short] = parts[-1]
    return result


def _scrape_metrics(http_port: int) -> dict:
    url = f"http://127.0.0.1:{http_port}/metrics"
    try:
        with urllib.request.urlopen(url, timeout=5) as resp:
            text = resp.read().decode("utf-8", errors="replace")
            return _parse_prometheus(text)
    except urllib.error.URLError as exc:
        return {"error": f"scrape failed: {exc}"}
    except OSError as exc:
        return {"error": f"scrape error: {exc}"}


# ---------------------------------------------------------------------------
# Actions
# ---------------------------------------------------------------------------

def _action_collect_logs(data_dirs: list, lines: int) -> dict:
    """
    data_dirs: list of paths returned by cluster start (same order as nodes).
    """
    result: dict = {}
    node_names = list(_NODES.keys())
    for i, data_dir in enumerate(data_dirs):
        node = node_names[i] if i < len(node_names) else f"broker-{i+1}"
        log_path = Path(data_dir) / "logs" / "broker.log"
        result[node] = _tail_lines(log_path, lines)
    return {"logs": result}


def _action_collect_metrics() -> dict:
    metrics: dict = {}
    for node, info in _NODES.items():
        metrics[node] = _scrape_metrics(info["http_port"])
    return {"metrics": metrics}


def _action_snapshot(data_dirs: list, lines: int) -> dict:
    logs_result = _action_collect_logs(data_dirs, lines)
    metrics_result = _action_collect_metrics()
    return {
        "snapshot_at": datetime.now(timezone.utc).isoformat(),
        "logs": logs_result["logs"],
        "metrics": metrics_result["metrics"],
    }


# ---------------------------------------------------------------------------
# Tool handler
# ---------------------------------------------------------------------------

def _observability_handler(args: dict, **_) -> str:
    action = args.get("action", "")
    data_dirs: list = args.get("data_dirs") or []
    lines: int = int(args.get("lines") or 100)

    try:
        if action == "collect_logs":
            if not data_dirs:
                return json.dumps({"error": "data_dirs is required for collect_logs"})
            result = _action_collect_logs(data_dirs, lines)

        elif action == "collect_metrics":
            result = _action_collect_metrics()

        elif action == "snapshot":
            if not data_dirs:
                return json.dumps({"error": "data_dirs is required for snapshot"})
            result = _action_snapshot(data_dirs, lines)

        else:
            result = {
                "error": (
                    f"unknown action: '{action}'. "
                    "Valid: collect_logs, collect_metrics, snapshot"
                )
            }
    except Exception as exc:
        logger.exception("observability: unhandled error in action '%s'", action)
        result = {"error": f"internal error: {exc}"}

    return json.dumps(result, ensure_ascii=False)


# ---------------------------------------------------------------------------
# Schema + registration
# ---------------------------------------------------------------------------

_SCHEMA: dict = {
    "name": "observability",
    "description": (
        "Collect logs and metrics from a running RobustMQ test cluster. "
        "collect_logs: tail last N lines from each broker's log file. "
        "collect_metrics: scrape Prometheus /metrics endpoint from each broker. "
        "snapshot: both in one call with a timestamp — use before and after "
        "fault injection for comparison."
    ),
    "parameters": {
        "type": "object",
        "properties": {
            "action": {
                "type": "string",
                "enum": ["collect_logs", "collect_metrics", "snapshot"],
                "description": (
                    "collect_logs: requires data_dirs. "
                    "collect_metrics: no extra args needed. "
                    "snapshot: requires data_dirs; returns logs + metrics + timestamp."
                ),
            },
            "data_dirs": {
                "type": "array",
                "items": {"type": "string"},
                "description": (
                    "List of broker data directories returned by cluster_manage(start). "
                    "Required for collect_logs and snapshot."
                ),
            },
            "lines": {
                "type": "integer",
                "description": "Number of log lines to tail per broker. Default: 100.",
                "default": 100,
            },
        },
        "required": ["action"],
    },
}

registry.register(
    name="observability",
    toolset="chaos",
    schema=_SCHEMA,
    handler=_observability_handler,
    emoji="🔭",
)
