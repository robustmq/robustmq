# 单机运行

本指南介绍如何在单台机器上通过二进制包启动 RobustMQ，适用于开发和测试环境。

## 安装

参考 [快速安装](../QuickGuide/Quick-Install.md) 完成安装，或使用一键安装脚本：

```bash
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
```

安装完成后，`robust-server`、`robust-ctl`、`robust-bench` 命令自动加入 PATH。

## 启动服务

```bash
robust-server start
```

默认使用 `config/server.toml`，也可以显式指定：

```bash
robust-server start config/server.toml
```

## 验证

**查看集群状态**

```bash
# 查看集群状态
robust-ctl cluster status

# 查看集群健康状态
robust-ctl cluster healthy

# 查看 MQTT 概览（连接数、订阅数等）
robust-ctl mqtt overview
```

`--server` 默认指向 `127.0.0.1:8080`，如需连接其他地址：

```bash
robust-ctl cluster status --server 192.168.1.10:8080
```

**MQTT 收发测试**

```bash
# 订阅（终端 1）
mqttx sub -h 127.0.0.1 -p 1883 -t "test/topic"

# 发布（终端 2）
mqttx pub -h 127.0.0.1 -p 1883 -t "test/topic" -m "Hello RobustMQ!"
```

收到消息即表示服务运行正常。Web 控制台：`http://localhost:3000`

## 停止服务

```bash
robust-server stop
```

## 默认端口

| 服务 | 端口 |
|------|------|
| MQTT | 1883 |
| HTTP API | 8083 |
| Placement Center gRPC | 1228 |
| Dashboard | 3000 |
