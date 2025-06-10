## 概述

RobustMQ提供了系统告警功能，用于监控和管理MQTT服务器的运行状态。
当系统内部状态发生异常时，RobustMQ会自动生成告警消息，并通过MQTT协议发布到特定的告警主题上。用户可以订阅这些主题，以便及时获取系统状态变化的信息。

当前RobustMQ的功能内容有以下部分：

- 监控系统的CPU和内存使用情况
- 获取和查询详细的告警信息
- 配置告警信息

通过系统告警，用户可以及时了解MQTT服务器的运行状态，快速响应潜在问题，确保系统的稳定性和可靠性。

## 当前支持的告警项

| 告警                  | 描述      |
|---------------------|---------|
| system_memory_usage | 系统内存使用率 |
| cpu_high_usage      | CPU低使用率 |
| cpu_low_usage       | CPU高使用率 |

## 获取告警信息

当前RobustMQ支持通过MQTT协议获取系统告警信息。用户可以订阅以下主题来接收告警消息：
`$SYS/brokers/${node}/alarms/activate`以及`$SYS/brokers/${node}/alarms/deactivate`。
其中`${node}`是节点的名称, `activate`是激活的告警，`deactivate`是未激活的告警。
告警消息的格式如下：

```json
{
  "name": "system_memory_usage",
  "message": "system_memory_usage is 80%, but config is 70%",
  "activate_at": 1700000000,
  "activated": false
}
```

## 告警项的配置

RobustMQ允许用户配置告警项的阈值和状态。用户可以通过修改配置文件或使用`Cli`来设置告警项的相关参数。

### 通过配置文件来进行配置

::: tip
⚠️注意: 当前暂时不支持设置定时检查CPU利用率和内存使用率的功能，默认现在`60S`一次。
:::

用户可以在RobustMQ的配置文件中设置告警项的阈值和状态。以下是一个示例配置：

```toml
[system_monitor]
enable = true
os_cpu_high_watermark = 70.0
os_cpu_low_watermark = 50.0
os_memory_high_watermark = 80.0
```

### 通过`Cli`来进行配置
