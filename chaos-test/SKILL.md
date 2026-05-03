---
name: robustmq-chaos-test
description: >
  7×24 chaos testing for RobustMQ. Injects broker-kill and network-delay faults,
  validates SDK client resilience across Python/Go/Rust/Java, and publishes a
  Markdown + JSON report to GitHub after each run.
requires_tools:
  - cluster_manage
  - observability
  - client
  - chaos
  - report
cron: "0 */4 * * *"
---

# RobustMQ Chaos Test Skill

## When to Use

**Cron trigger** (every 4 hours): the system message will say
> "按 P0 跑一轮 RobustMQ 故障场景"

**Manual CLI**: user says something like
> "帮我跑一轮 RobustMQ chaos 测试" / "run a chaos test round"

In both cases execute the **Full Run** below.

If the user says "按 P1 跑一轮" or names a specific scenario, execute the
**Single Scenario** flow for that scenario only.

---

## Pre-check

Before starting any run:

1. Call `cluster_manage(action=status)`.
   - If `status` is NOT `stopped`, call `cluster_manage(action=stop)` to clear
     any leftover state from a previous run.
2. Verify `ROBUSTMQ_HOME` is set (the cluster tool fails fast if it is not —
   surface that error immediately and stop).

---

## Scenario Catalogue

| Scenario name | fault_type | target | params | Core? |
|---|---|---|---|---|
| broker-kill-single | broker-kill | robustmq-server | — | ✅ |
| network-delay-100ms | network-delay | eth0 | delay_ms=100, jitter_ms=10 | — |
| leader-transfer | broker-kill | robustmq-server | — | ✅ |

> **Note:** Update this table when new scenarios are added. Target names and
> interface names depend on the deployment environment — verify before running.

Core scenarios: **broker-kill-single**, **leader-transfer**.
Run passed = all core pass AND non-core pass rate ≥ 75%.

---

## Single Scenario — 5-Step Flow

Execute these steps sequentially. Do NOT skip steps.

### Step 1 — Baseline Snapshot

```
observability(action=snapshot, data_dirs=<from cluster start>)
```

Record the snapshot as `baseline`. Proceed even if some metrics are unavailable;
log a warning but do not abort.

### Step 2 — Inject Fault

```
chaos(action=inject, fault_type=<type>, target=<target>, params=<params>)
```

Save the returned `fault_id`. If inject returns an error, mark the scenario
`passed=False` with `status=inject_error` and skip to Step 5 (skip recover).

### Step 3 — Fault-Period SDK Observation (record only)

```
client(action=run, scenario=<scenario>, cluster_endpoint=<endpoint>)
```

Record all results. **Do NOT use these results to determine pass/fail.**
Their only purpose is observability — they show what clients experienced
during the fault. A high loss rate here is expected and normal.

### Step 4 — Recover

```
chaos(action=recover, fault_id=<fault_id>)
```

If recover returns an error, log it and continue — attempt self-healing
validation anyway.

### Step 5 — Self-Healing Validation (sole pass/fail basis)

Wait **60 seconds** after recovery, then run:

```
client(action=run, scenario=<scenario>, cluster_endpoint=<endpoint>)
```

**Pass criteria** (ALL must hold):
- `exit_code == 0`
- `lost == 0`
- `p99_ms < 500`

If any criterion fails → scenario `passed=False`.
If `status=script_format_error` → scenario `passed=False`, note the format error
separately (this is a test-infrastructure issue, not a RobustMQ bug).

---

## Full Run Flow

1. **Pre-check** (see above).
2. **Start cluster**: `cluster_manage(action=start)` → save `endpoint` and `data_dirs`.
3. **Run each scenario** using the Single Scenario flow.
   - Run scenarios sequentially (not in parallel) to avoid interference.
   - If a scenario crashes the cluster (all brokers dead), restart it before
     continuing: `cluster_manage(action=stop)` → `cluster_manage(action=start)`.
4. **Stop cluster**: `cluster_manage(action=stop)`.
5. **Generate report**: `report(action=generate_and_push, run_data={...})`.
   - `run_data` must include: `run_id`, `started_at`, `finished_at`, `scenarios`.
   - Each scenario entry: scenario name, sdk, passed, sent, received, lost,
     p99_ms, duration_seconds, errors, status.
6. **Send Feishu notification**:
   - If `run_passed=True`: send brief pass message with `github_url`.
   - If `run_passed=False`: send failure alert listing failed scenarios and `github_url`.
   - If `consecutive_failures >= 3`: prepend `🚨 连续 {n} 轮失败，请人工介入`.

---

## Circuit Breaker

Track `consecutive_failures` across runs (persist in your memory or state):

- Increment on `run_passed=False`.
- Reset to 0 on `run_passed=True`.
- If `consecutive_failures >= 3`: send an urgent Feishu alert and **pause**
  the cron schedule. Do NOT continue running automatically until a human
  acknowledges and resets the counter.

---

## Feishu Message Templates

**Pass:**
```
✅ RobustMQ 故障测试通过
Run ID: {run_id}  时间: {finished_at}
核心场景: 全部通过  总通过率: {pass_rate}%
报告: {github_url}
```

**Fail:**
```
❌ RobustMQ 故障测试失败
Run ID: {run_id}  时间: {finished_at}
失败场景: {failed_scenario_list}
报告: {github_url}
```

**Circuit breaker:**
```
🚨 连续 {consecutive_failures} 轮失败，请人工介入
最后失败: {run_id}  {finished_at}
报告: {github_url}
```

---

## Pitfalls

- **Never judge pass/fail on fault-period results** (Step 3). Only Step 5
  post-recovery validation counts.
- **`script_format_error` ≠ RobustMQ bug.** Report it separately; do not
  inflate the failure count. Fix the script first.
- **ROBUSTMQ_HOME must be set** before any run. The cluster tool returns an
  error immediately if it is not — surface it and stop rather than retrying.
- **Consecutive failures count whole runs**, not individual scenarios.
  One run with two failed scenarios = 1 failure, not 2.
- **eth0 is not universal.** The network-delay target interface name varies by
  host. Verify it before running in a new environment.
- **Deploy Key permissions.** If `report` returns `push_error`, the reports are
  still written locally at `json_path` / `markdown_path`. Investigate the key
  before declaring the run lost.
- **60-second wait is mandatory.** Do not skip or shorten it — RobustMQ leader
  election and connection re-establishment take time.
