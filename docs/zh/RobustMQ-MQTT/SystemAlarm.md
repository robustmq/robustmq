## 概述

RobustMQ提供了系统告警功能，用于监控和管理MQTT服务器的运行状态。
当系统内部状态发生异常时，RobustMQ会自动生成告警消息，并通过MQTT协议发布到特定的告警主题上。用户可以订阅这些主题，以便及时获取系统状态变化的信息。

当前RobustMQ的功能内容有以下部分：

- 监控每个节点上 Broker 进程的 CPU 和内存使用情况
- 获取和查询详细的告警信息
- 配置告警信息

通过系统告警，用户可以及时了解MQTT服务器的运行状态，快速响应潜在问题，确保系统的稳定性和可靠性。

## 当前支持的告警项

| 告警                  | 描述      |
|---------------------|---------|
| HighMemoryUsage | 当前节点的 Broker 进程内存使用率超过 `os_memory_high_watermark` |
| HighCpuUsage | 当前节点的 Broker 进程 CPU 使用率超过 `os_cpu_high_watermark` |

## 获取告警信息

当当前节点的 Broker 进程 CPU 或内存使用率超过配置的高水位线时，RobustMQ
会向固定主题 `$SYS/brokers/alarms/alert` 发布告警。旧文档中的
`${node}` 主题路径形式不是当前实现实际发布的形式。告警仍通过统一封装中的
`node_id` 和 `node_ip` 标识受影响的节点，因此消费者可以确定具体机器。
主题只报告每次检查中超过配置阈值的结果。每条活动告警的
`activated` 都是 `true`。当使用率达到或低于阈值时，该次检查不会发布告警；
不会发布 `activated: false` 的恢复消息，也没有单独的 deactivate 或 clear 主题。

### 当前设计决策

告警激活状态直接根据每次 CPU/内存采样计算：`current_usage > 配置水位`。
系统不保存内存中的告警状态。只要使用率持续超过阈值，每 60 秒检查就发布一条
活动告警；达到或低于阈值时，该次检查不发布消息。这样监控逻辑保持无状态，
Broker 重启后下一次采样即可按正常流程处理。

如果 Broker 无法将活动告警写入内部系统主题，下一次 60 秒检查会自然重试。
这里确认的是 Broker 内部写入成功，不代表每个 MQTT 客户端都返回了投递确认。

当前每次活动告警仍会持久化一条事件记录，以支持现有的
`/api/mqtt/system-alarm/list` 接口和 `robust-ctl mqtt system-alarm list` 命令
查看历史。若后续改为完全不持久化
告警，就需要同时移除或重新设计现有历史查询接口和 CLI，这是单独的后续决策。

```json
{
  "node_id": 1,
  "node_ip": "127.0.0.1",
  "ts": 1749774914000,
  "value": {
    "name": "HighMemoryUsage",
    "message": "HighMemoryUsage is 85.2%, but config is 80%",
    "activate_at": 1749774914,
    "activated": true
  }
}
```

以上是包含实际字段和值的示例消息。`node_id`、`node_ip`、使用率和时间戳
会根据产生告警的 Broker 实例而变化。

MQTT 封装消息包含用于标识产生告警机器的字段；持久化后的告警/API 记录也会
在记录顶层提供相同字段，方便历史排查：

| 字段 | 含义 |
|------|------|
| `node_id` | 执行告警检查并发布告警的 RobustMQ 节点 ID |
| `node_ip` | 该 Broker 节点配置的 `broker_ip`；未配置时使用本机 IP 回退值 |
| `ts` | 系统主题封装消息的生成时间戳 |
| `value` | 告警详情，包括告警名称、消息、激活时间和激活状态 |

判断具体哪台机器需要处理时，应使用 `node_id` 作为稳定的节点身份，使用
`node_ip` 作为网络地址。这些字段由发布告警的 Broker 自动生成，不是由
MQTT 客户端提供的。

## 告警项的配置

RobustMQ允许用户配置告警项的阈值和状态。用户可以通过修改配置文件或使用`Cli`来设置告警项的相关参数。

### 通过配置文件来进行配置

系统监控默认关闭。启用后，告警循环每 60 秒检查一次 CPU 和内存使用率，
可以使用以下配置：

```toml
[mqtt_runtime.system_monitor]
enable = true
os_cpu_high_watermark = 70.0
os_memory_high_watermark = 80.0
```

### 通过`Cli`来进行配置

可以使用 CLI 查看已持久化的告警记录：

```bash
robust-ctl mqtt system-alarm list
```

一条内存告警记录持久化后的示例输出：

```text
system alarm list result:
+---------+-------------+-----------------+------------------------------------------------+-------------+-----------+
| node_id | node_ip     | name            | message                                        | create_time | activated |
+=========+=============+=================+================================================+=============+===========+
| 1       | 127.0.0.1   | HighMemoryUsage | HighMemoryUsage is 85.2%, but config is 80%    | 1749774914  | true      |
+---------+-------------+-----------------+------------------------------------------------+-------------+-----------+
```
