"""
chaos.py — fault injection and recovery for RobustMQ chaos testing.

Strategy (per technical spec):
  Chaosd HTTP API  — faults that need precise measurement/replay:
                     process kill signals, network delay distribution,
                     disk I/O throttle, clock skew.
  tc / kill        — faults that only need reachability validation.

First-version implemented faults:
  broker-kill     — Chaosd process attack (SIGKILL)
  network-delay   — Chaosd network attack with configurable delay/jitter

Not yet implemented (returns {"error": "not_implemented"}):
  network-partition / disk-fill / cpu-stress / clock-skew

Chaosd endpoint is read from config.yml — never hardcoded.

Config file: ~/.hermes/skills/robustmq-chaos-test/config.yml
Expected keys:
  chaosd:
    endpoint: "http://127.0.0.1:31767"   # Chaosd daemon HTTP port

Fault records are persisted to disk so recover survives session restarts.
Store: ~/.hermes/skills/robustmq-chaos-test/faults/<fault_id>.json
"""

import json
import logging
import os
import urllib.error
import urllib.request
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

from tools.registry import registry

logger = logging.getLogger(__name__)

_NOT_IMPLEMENTED = frozenset({
    "network-partition",
    "disk-fill",
    "cpu-stress",
    "clock-skew",
})

_CONFIG_PATH = (
    Path.home() / ".hermes" / "skills" / "robustmq-chaos-test" / "config.yml"
)
_FAULT_DIR = (
    Path.home() / ".hermes" / "skills" / "robustmq-chaos-test" / "faults"
)


# ---------------------------------------------------------------------------
# Config loader
# ---------------------------------------------------------------------------

def _load_chaosd_endpoint() -> Optional[str]:
    """Read chaosd.endpoint from config.yml. Returns None on any error."""
    if not _CONFIG_PATH.exists():
        return None
    try:
        # Minimal YAML parsing — only reads the chaosd.endpoint key.
        # Avoids a PyYAML import that may not be available.
        text = _CONFIG_PATH.read_text(encoding="utf-8")
        in_chaosd = False
        for line in text.splitlines():
            stripped = line.strip()
            if stripped.startswith("chaosd:"):
                in_chaosd = True
                continue
            if in_chaosd:
                if stripped.startswith("endpoint:"):
                    value = stripped.split(":", 1)[1].strip().strip('"').strip("'")
                    return value if value else None
                if stripped and not stripped.startswith("#") and ":" in stripped and not line.startswith(" "):
                    in_chaosd = False
    except OSError as exc:
        logger.error("chaos: failed to read config.yml: %s", exc)
    return None


# ---------------------------------------------------------------------------
# Fault record helpers
# ---------------------------------------------------------------------------

def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def _save_fault(record: dict) -> None:
    _FAULT_DIR.mkdir(parents=True, exist_ok=True)
    (_FAULT_DIR / f"{record['fault_id']}.json").write_text(
        json.dumps(record, indent=2), encoding="utf-8"
    )


def _load_fault(fault_id: str) -> Optional[dict]:
    path = _FAULT_DIR / f"{fault_id}.json"
    if not path.exists():
        return None
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        logger.error("chaos: failed to load fault %s: %s", fault_id, exc)
        return None


# ---------------------------------------------------------------------------
# Chaosd HTTP client
# ---------------------------------------------------------------------------

def _chaosd_post(endpoint: str, path: str, payload: dict) -> dict:
    url = endpoint.rstrip("/") + path
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        url, data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            return json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        return {"error": f"Chaosd HTTP {exc.code}: {body[:300]}"}
    except urllib.error.URLError as exc:
        return {"error": f"Chaosd unreachable: {exc.reason}"}
    except OSError as exc:
        return {"error": f"Chaosd request error: {exc}"}


def _chaosd_delete(endpoint: str, path: str) -> dict:
    url = endpoint.rstrip("/") + path
    req = urllib.request.Request(url, method="DELETE")
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            return json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        return {"error": f"Chaosd HTTP {exc.code}: {body[:300]}"}
    except urllib.error.URLError as exc:
        return {"error": f"Chaosd unreachable: {exc.reason}"}
    except OSError as exc:
        return {"error": f"Chaosd request error: {exc}"}


# ---------------------------------------------------------------------------
# Fault implementations
# ---------------------------------------------------------------------------

def _inject_broker_kill(endpoint: str, target: str, params: dict, fault_id: str) -> dict:
    """
    Chaosd process attack — send SIGKILL to a process matching target.
    target: process name or command pattern, e.g. "robustmq-server"
    """
    payload = {
        "action": "kill",
        "process": target,
        "signal": 9,
    }
    resp = _chaosd_post(endpoint, "/api/attack/process", payload)
    if "error" in resp:
        return resp

    chaosd_uid = resp.get("uid") or resp.get("id") or ""
    record = {
        "fault_id": fault_id,
        "fault_type": "broker-kill",
        "target": target,
        "chaosd_uid": chaosd_uid,
        "injected_at": _now_iso(),
        "status": "active",
        "endpoint": endpoint,
    }
    _save_fault(record)
    return {
        "fault_id": fault_id,
        "fault_type": "broker-kill",
        "target": target,
        "chaosd_uid": chaosd_uid,
        "injected_at": record["injected_at"],
        "status": "active",
    }


def _inject_network_delay(endpoint: str, target: str, params: dict, fault_id: str) -> dict:
    """
    Chaosd network attack — add latency to traffic from/to a device.
    target:             network interface, e.g. "eth0"
    params.delay_ms:    mean delay in milliseconds (default 100)
    params.jitter_ms:   jitter in milliseconds (default 10)
    params.correlation: correlation % (default 0)
    """
    delay_ms = int(params.get("delay_ms") or 100)
    jitter_ms = int(params.get("jitter_ms") or 10)
    correlation = int(params.get("correlation") or 0)

    payload = {
        "action": "delay",
        "device": target,
        "latency": f"{delay_ms}ms",
        "jitter": f"{jitter_ms}ms",
        "correlation": str(correlation),
    }
    resp = _chaosd_post(endpoint, "/api/attack/network", payload)
    if "error" in resp:
        return resp

    chaosd_uid = resp.get("uid") or resp.get("id") or ""
    record = {
        "fault_id": fault_id,
        "fault_type": "network-delay",
        "target": target,
        "chaosd_uid": chaosd_uid,
        "params": {"delay_ms": delay_ms, "jitter_ms": jitter_ms, "correlation": correlation},
        "injected_at": _now_iso(),
        "status": "active",
        "endpoint": endpoint,
    }
    _save_fault(record)
    return {
        "fault_id": fault_id,
        "fault_type": "network-delay",
        "target": target,
        "chaosd_uid": chaosd_uid,
        "params": record["params"],
        "injected_at": record["injected_at"],
        "status": "active",
    }


# ---------------------------------------------------------------------------
# Actions
# ---------------------------------------------------------------------------

def _action_inject(fault_type: str, target: str,
                   duration_seconds: int, params: dict) -> dict:
    if fault_type in _NOT_IMPLEMENTED:
        return {"error": f"not_implemented: '{fault_type}' will be added in a future version"}

    endpoint = _load_chaosd_endpoint()
    if not endpoint:
        return {
            "error": (
                "Chaosd endpoint not configured. "
                "Add chaosd.endpoint to "
                "~/.hermes/skills/robustmq-chaos-test/config.yml"
            )
        }

    fault_id = str(uuid.uuid4())

    if fault_type == "broker-kill":
        return _inject_broker_kill(endpoint, target, params, fault_id)
    if fault_type == "network-delay":
        return _inject_network_delay(endpoint, target, params, fault_id)

    return {"error": f"unknown fault_type: '{fault_type}'"}


def _action_recover(fault_id: str) -> dict:
    record = _load_fault(fault_id)
    if record is None:
        return {"error": f"fault record not found: '{fault_id}'"}

    if record.get("status") == "recovered":
        return {
            "fault_id": fault_id,
            "status": "already_recovered",
            "recovered_at": record.get("recovered_at"),
        }

    endpoint = record.get("endpoint", "")
    chaosd_uid = record.get("chaosd_uid", "")

    if not endpoint or not chaosd_uid:
        return {"error": f"fault record is missing endpoint or chaosd_uid: {record}"}

    resp = _chaosd_delete(endpoint, f"/api/attack/{chaosd_uid}")
    if "error" in resp:
        return resp

    record["status"] = "recovered"
    record["recovered_at"] = _now_iso()
    _save_fault(record)
    return {
        "fault_id": fault_id,
        "fault_type": record.get("fault_type"),
        "status": "recovered",
        "recovered_at": record["recovered_at"],
    }


# ---------------------------------------------------------------------------
# Tool handler
# ---------------------------------------------------------------------------

def _chaos_handler(args: dict, **_) -> str:
    action = args.get("action", "")
    try:
        if action == "inject":
            fault_type = args.get("fault_type") or ""
            target = args.get("target") or ""
            duration_seconds = int(args.get("duration_seconds") or 0)
            params = args.get("params") or {}
            if not fault_type:
                result = {"error": "fault_type is required for inject"}
            elif not target:
                result = {"error": "target is required for inject"}
            else:
                result = _action_inject(fault_type, target, duration_seconds, params)

        elif action == "recover":
            fault_id = args.get("fault_id") or ""
            if not fault_id:
                result = {"error": "fault_id is required for recover"}
            else:
                result = _action_recover(fault_id)

        else:
            result = {"error": f"unknown action: '{action}'. Valid: inject, recover"}

    except Exception as exc:
        logger.exception("chaos: unhandled error in action '%s'", action)
        result = {"error": f"internal error: {exc}"}

    return json.dumps(result, ensure_ascii=False)


# ---------------------------------------------------------------------------
# Schema + registration
# ---------------------------------------------------------------------------

_SCHEMA: dict = {
    "name": "chaos",
    "description": (
        "Inject or recover chaos faults via Chaosd HTTP API. "
        "Implemented: broker-kill (process SIGKILL), network-delay (tc netem via Chaosd). "
        "Not implemented yet: network-partition, disk-fill, cpu-stress, clock-skew. "
        "Chaosd endpoint is read from config.yml — never hardcoded. "
        "Each inject returns a fault_id; pass it to recover to undo."
    ),
    "parameters": {
        "type": "object",
        "properties": {
            "action": {
                "type": "string",
                "enum": ["inject", "recover"],
                "description": (
                    "inject: apply the fault; returns fault_id. "
                    "recover: undo the fault by fault_id."
                ),
            },
            "fault_type": {
                "type": "string",
                "enum": [
                    "broker-kill",
                    "network-delay",
                    "network-partition",
                    "disk-fill",
                    "cpu-stress",
                    "clock-skew",
                ],
                "description": "Required for action=inject.",
            },
            "target": {
                "type": "string",
                "description": (
                    "Required for action=inject. "
                    "broker-kill: process name or pattern, e.g. 'robustmq-server'. "
                    "network-delay: network interface name, e.g. 'eth0'."
                ),
            },
            "duration_seconds": {
                "type": "integer",
                "description": (
                    "Informational hint for fault duration. "
                    "Recovery must still be called explicitly. Default: 0."
                ),
                "default": 0,
            },
            "params": {
                "type": "object",
                "description": (
                    "Fault-specific parameters. "
                    "network-delay: {delay_ms: 100, jitter_ms: 10, correlation: 0}."
                ),
            },
            "fault_id": {
                "type": "string",
                "description": "Required for action=recover.",
            },
        },
        "required": ["action"],
    },
}

registry.register(
    name="chaos",
    toolset="chaos",
    schema=_SCHEMA,
    handler=_chaos_handler,
    emoji="💥",
)
