# 体验 RobustMQ Kafka

## 前提：启动 Broker

参考 [快速安装](Quick-Install.md) 完成安装，然后启动服务：

```bash
robust-server start
```

Kafka 协议随 RobustMQ 一起启动，默认监听端口 `9092`，无需额外配置——**与 MQTT、NATS、mq9、AMQP 是同一个 Broker 进程**，一条命令启动后所有协议同时就绪。

---

## 准备 Kafka 客户端

选择以下任意一种方式测试 Kafka 生产消费。

### 方式一：官方 Kafka CLI

使用官方发行版自带的 `kafka-*.sh` 命令行工具（任意 3.x / 4.x 版本即可），无需额外安装 RobustMQ 专用工具。

### 方式二：RobustMQ 自带 robust-bench

`robust-bench` 是 RobustMQ 内置的压测工具，随安装包一起提供。详细使用说明参考：[Bench CLI 文档](../Bench/Bench-CLI.md)

---

## 建 Topic、生产与消费

```bash
# 建 topic
kafka-topics.sh --bootstrap-server localhost:9092 \
  --create --topic quickstart --partitions 3

# 生产
kafka-console-producer.sh --bootstrap-server localhost:9092 --topic quickstart
>hello
>robustmq kafka
# Ctrl-C 结束

# 消费
kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic quickstart --from-beginning
```

能看到刚才生产的两行，即表示数据面工作正常。

> RobustMQ 默认开启自动创建 topic，直接生产也会隐式建 topic；显式创建更可控。

### 用消费组消费

```bash
kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic quickstart --group g1 --from-beginning

kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --describe --group g1
```

更多命令见 [CLI 操作指南](../RobustMQ-Kafka/CLI-Guide.md)。

### 压测（robust-bench）

```bash
robust-bench kafka pub --count 100 --duration-secs 30
robust-bench kafka sub --count 100 --duration-secs 30
```

更多参数参考 [Bench CLI 文档](../Bench/Bench-CLI.md)。

---

## SDK 接入

RobustMQ 兼容标准 Kafka 协议，使用任意社区标准 Kafka 客户端库即可直接接入。

```java
Properties props = new Properties();
props.put("bootstrap.servers", "localhost:9092");
props.put("key.serializer", "org.apache.kafka.common.serialization.StringSerializer");
props.put("value.serializer", "org.apache.kafka.common.serialization.StringSerializer");

try (KafkaProducer<String, String> producer = new KafkaProducer<>(props)) {
    producer.send(new ProducerRecord<>("quickstart", "k1", "hello from java"));
    producer.flush();
}
```

## 下一步

- [核心概念](../RobustMQ-Kafka/KafkaCoreConcepts.md) — 理解 Topic / Offset / 消费组 / Coordinator
- [快速开始（完整版）](../RobustMQ-Kafka/QuickStart.md) — 含 SASL 连接、Java 消费者示例
- [协议兼容矩阵](../RobustMQ-Kafka/Protocol.md) — 你的客户端能用哪些 API
- [兼容性与限制](../RobustMQ-Kafka/Compatibility-and-Limitations.md)
