# 体验 RobustMQ MQTT

## 前提：启动 Broker

参考 [快速安装](Quick-Install.md) 完成安装，然后启动服务：

```bash
robust-server start
```

启动成功后验证状态：

```bash
robust-ctl status
```

---

## 准备 MQTT 客户端

选择以下任意一种方式测试 MQTT 收发消息。

### 方式一：MQTTX CLI

MQTTX CLI 是 EMQ 开源的 MQTT 命令行客户端，安装文档参考：[https://mqttx.app/zh/docs/cli](https://mqttx.app/zh/docs/cli)

安装完成后即可使用 `mqttx` 命令。

### 方式二：RobustMQ 自带 robust-bench

`robust-bench` 是 RobustMQ 内置的压测工具，随安装包一起提供，无需额外安装。详细使用说明参考：[Bench CLI 文档](../Bench/Bench-CLI.md)

---

## 发布与订阅

### 使用 MQTTX

打开两个终端，分别运行：

```bash
# 终端 1：订阅
mqttx sub -h localhost -p 1883 -t "test/topic"

# 终端 2：发布
mqttx pub -h localhost -p 1883 -t "test/topic" -m "Hello RobustMQ!"
```

订阅端收到消息则说明 MQTT 收发正常。

常用参数：

```bash
# 指定 QoS
mqttx pub -h localhost -p 1883 -t "test/qos1" -m "msg" -q 1

# 发布保留消息
mqttx pub -h localhost -p 1883 -t "test/retained" -m "retained msg" -r

# 通配符订阅
mqttx sub -h localhost -p 1883 -t "test/#"
```

### 使用 robust-bench

```bash
# 连接压测（建立 100 个连接）
robust-bench mqtt conn --count 100

# 发布压测（100 个客户端，持续 30 秒）
robust-bench mqtt pub --count 100 --duration-secs 30

# 订阅压测（100 个客户端订阅）
robust-bench mqtt sub --count 100 --duration-secs 30
```

更多参数和用法参考 [Bench CLI 文档](../Bench/Bench-CLI.md)。
