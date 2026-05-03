# p0-broker-kill-leader

**协议:** MQTT  
**优先级:** P0 — 每次触发都跑，不可跳过  
**预计耗时:** ~3 分钟（注入 30s + 健康等待 + 60s 自愈窗口 + SDK 矩阵验证）

---

## 场景描述

在 3 节点 RobustMQ 集群中，broker-1 承担 Leader 角色。本场景模拟 Leader
节点进程被 SIGKILL 的情况，验证集群能否在 broker-1 恢复后重新接受连接，
SDK 客户端能否自愈并在 60 秒窗口内实现零丢包。

---

## 执行步骤（对照 SKILL.md 单场景五步）

### Step 1 — 基线 Snapshot

调用 `observability` Tool，记录故障前集群状态：

```
observability(action=snapshot, data_dirs=<来自 cluster_manage start 的 data_dirs>)
```

把返回的 snapshot 标记为 `baseline`，存入本轮 run_data。

### Step 2 — 注入故障

调用 `chaos` Tool 向 broker-1 发送 SIGKILL：

```
chaos(
  action=inject,
  fault_type=broker-kill,
  target=robustmq-server,
  duration_seconds=30,
  params={}
)
```

保存返回的 `fault_id`，后续 recover 需要它。

> broker-kill 的 target 是进程名，不是节点名。Chaosd 会匹配系统中第一个
> 命中 `robustmq-server` 的进程，即 broker-1（端口 1883）。

### Step 3 — 故障期间 SDK 观测（只记录，不判断）

注入成功后立刻调用 `client` Tool，使用 **config.yml 中 P0 档 SDK 列表**
跑 `basic-pubsub` 场景：

```
client(
  action=run,
  scenario=basic-pubsub,
  cluster_endpoint=127.0.0.1:1883,
  sdk=<从 config.yml p0.sdk_matrix 逐个取，或 omit 跑全部>
)
```

**这段结果只记录，不作为 pass/fail 依据。** 故障期间出现丢包、连接失败是
预期行为。把返回的 sent/received/lost/p99_ms/errors 写入 run_data 的
`fault_period_observations` 字段。

### Step 4 — 恢复故障

30 秒故障窗口结束后，调用 `chaos` Tool 撤销攻击：

```
chaos(action=recover, fault_id=<step2 返回的 fault_id>)
```

recover 成功只代表 Chaosd 停止攻击配置，**不代表 broker-1 进程已重启**。
执行下一步前必须先等 broker-1 健康检查通过。

**等待 broker-1 健康：** 轮询 `http://127.0.0.1:1883/health`，每 2 秒一次，
最多等 60 秒。健康检查返回 HTTP 200 后，再额外等 **60 秒**，让集群完成
Leader 重新选举和 SDK 重连。

如果 60 秒内健康检查始终不通过，记录 `status=broker_not_recovered`，
跳过 Step 5，场景标记为失败。

### Step 5 — 自愈验证（pass/fail 唯一依据）

60 秒等待结束后，用完整 P0 SDK 矩阵再跑一轮 `basic-pubsub`：

```
client(
  action=run,
  scenario=basic-pubsub,
  cluster_endpoint=127.0.0.1:1883
)
```

**通过标准（全部满足）：**

| 字段 | 要求 |
|------|------|
| exit_code | == 0 |
| lost | == 0 |
| p99_ms | < 500 |

任意一条不满足 → 场景失败。`status=script_format_error` 也算失败，
但要在 report 里单独标注，区分"测试基础设施问题"和"RobustMQ 行为异常"。

---

## 失败处理

场景失败时：

1. 将失败结果写入 run_data.scenarios，`passed=false`，保留完整的
   sent/lost/p99_ms/errors/status 字段。
2. 打一条 observability snapshot（标记为 `post_failure`）。
3. **继续执行下一个场景**，不中断整轮 run。
4. 整轮结束后由 `report` Tool 的 `_compute_run_passed` 决定 run 是否通过。
   本场景是核心场景，失败会直接导致 `run_passed=false`。

---

## 参数来源

| 参数 | 来源 |
|------|------|
| SDK 列表 | `config.yml` → `p0.sdk_matrix` |
| SDK 版本 | `config.yml` → 各 SDK 的 `version` 字段 |
| Chaosd 端点 | `config.yml` → `chaosd.endpoint` |
| 报告仓库 | `config.yml` → `reports.repo_url` |
| broker-1 健康检查 URL | 固定：`http://127.0.0.1:1883/health` |
