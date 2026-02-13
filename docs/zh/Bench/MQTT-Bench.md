# MQTT Bench 使用文档

本文聚焦 `robust-bench mqtt` 的使用与参数说明。

## 1. 子命令总览

```bash
robust-bench mqtt conn ...
robust-bench mqtt pub ...
robust-bench mqtt sub ...
```

## 2. conn：连接压测

### 用途

评估 Broker 的连接建立能力和稳定连接规模。

### 常用参数

- `--count`：连接客户端总数
- `--interval-ms`：连接创建间隔
- `--mode`：`create|hold`
- `--hold-secs`：仅 `hold` 模式有效，表示持连接时长（秒）

### 示例

```bash
robust-bench mqtt conn \
  --host 127.0.0.1 \
  --port 1883 \
  --username admin \
  --password robustmq \
  --count 50000 \
  --interval-ms 1 \
  --mode hold \
  --hold-secs 90
```

## 3. pub：发布压测

### 用途

评估发布吞吐、发布成功率、发布延迟。

### 参数

- `--topic`：主题模板，支持 `%i` 占位符
- `--payload-size`：消息体大小（字节）
- `--message-interval-ms`：单客户端发送间隔
- `--qos`：QoS 等级

### 示例

```bash
robust-bench mqtt pub \
  --host 127.0.0.1 \
  --port 1883 \
  --username admin \
  --password robustmq \
  --count 2000 \
  --topic load/%i \
  --payload-size 512 \
  --message-interval-ms 5 \
  --qos 0 \
  --duration-secs 120
```

## 4. sub：订阅压测

### 用途

评估接收吞吐、接收稳定性和订阅侧超时情况。

### 参数

- `--topic`：订阅主题（可含通配符）
- `--count`：订阅客户端数量
- `--qos`：QoS 等级

### 示例

```bash
robust-bench mqtt sub \
  --host 127.0.0.1 \
  --port 1883 \
  --username admin \
  --password robustmq \
  --count 5000 \
  --topic "load/#" \
  --qos 1 \
  --duration-secs 120
```

## 5. 结果解读建议

- `avg_ops_per_sec`：平均吞吐能力
- `peak_ops_per_sec`：峰值吞吐能力
- `success_rate(%)`：成功率，优先关注是否接近 100%
- `latency_p95/p99`：尾延迟，压测最关键指标
- `Error Distribution`：定位失败原因的第一入口

## 6. 标准压测流程建议

1. 先执行 `conn`（验证连接上限）
2. 再执行 `pub` + `sub`（验证数据面能力）
3. 固定参数重复 3 次，取中位值作为基线
