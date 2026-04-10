# 二进制运行

本指南介绍如何通过二进制包运行 RobustMQ，包括单机模式（开发/测试）和集群模式（生产）。

> 没有现成的二进制包？可以[在本地编译打包](../ContributionGuide/ContributingCode/Build-and-Package.md)。

## 安装

使用一键安装脚本：

```bash
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
```

也可参考 [快速安装](../QuickGuide/Quick-Install.md) 了解更多安装方式。

安装完成后，`robust-server`、`robust-ctl`、`robust-bench` 命令自动加入 PATH。

---

## 单机模式

适用于开发和测试环境。

### 启动

```bash
robust-server start
```

默认使用 `config/server.toml`，也可以显式指定：

```bash
robust-server start config/server.toml
```

### 验证

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

### 停止

```bash
robust-server stop
```

### 默认端口

| 服务 | 端口 |
|------|------|
| MQTT | 1883 |
| HTTP API | 8083 |
| Placement Center gRPC | 1228 |
| Dashboard | 3000 |

---

## 集群模式

适用于高可用场景。安装包内已包含集群配置模板：

```
config/cluster/
├── server-1.toml   # 节点 1（grpc: 1128，mqtt: 1883）
├── server-2.toml   # 节点 2（grpc: 1228，mqtt: 2883）
└── server-3.toml   # 节点 3（grpc: 1328，mqtt: 3883）
```

### 场景一：单机三节点（开发测试）

三份配置文件默认均指向 `127.0.0.1`，直接启动即可：

```bash
# 分别在三个终端中执行
robust-server start config/cluster/server-1.toml
robust-server start config/cluster/server-2.toml
robust-server start config/cluster/server-3.toml
```

### 场景二：多机集群（生产部署）

假设三台机器 IP 分别为 `10.0.0.1`、`10.0.0.2`、`10.0.0.3`，在每台机器上安装后，修改对应配置文件中的 `broker_ip` 和 `meta_addrs`：

```toml
# 节点 1（10.0.0.1）修改 config/cluster/server-1.toml 中：
broker_ip = "10.0.0.1"
meta_addrs = { 1 = "10.0.0.1:1128", 2 = "10.0.0.2:1228", 3 = "10.0.0.3:1328" }

# 节点 2（10.0.0.2）修改 config/cluster/server-2.toml 中：
broker_ip = "10.0.0.2"
meta_addrs = { 1 = "10.0.0.1:1128", 2 = "10.0.0.2:1228", 3 = "10.0.0.3:1328" }

# 节点 3（10.0.0.3）修改 config/cluster/server-3.toml 中：
broker_ip = "10.0.0.3"
meta_addrs = { 1 = "10.0.0.1:1128", 2 = "10.0.0.2:1228", 3 = "10.0.0.3:1328" }
```

然后在各节点上启动：

```bash
# 节点 1
robust-server start config/cluster/server-1.toml

# 节点 2
robust-server start config/cluster/server-2.toml

# 节点 3
robust-server start config/cluster/server-3.toml
```

### 验证

```bash
# 查看集群状态（连接任意节点）
robust-ctl cluster status --server 10.0.0.1:8080

# 查看集群健康状态
robust-ctl cluster healthy --server 10.0.0.1:8080

# 查看 MQTT 概览
robust-ctl mqtt overview --server 10.0.0.1:8080
```

**MQTT 跨节点收发测试**

```bash
# 订阅（连接节点 1）
mqttx sub -h 10.0.0.1 -p 1883 -t "test/cluster"

# 发布（连接节点 2）
mqttx pub -h 10.0.0.2 -p 1883 -t "test/cluster" -m "Hello Cluster!"
```

节点 1 收到消息即表示集群运行正常。

### 停止

```bash
robust-server stop
```

### 默认端口（各节点）

| | 节点 1 | 节点 2 | 节点 3 |
|-|--------|--------|--------|
| **MQTT TCP** | 1883 | 2883 | 3883 |
| **gRPC** | 1128 | 1228 | 1328 |
| **HTTP API** | 8080 | 8082 | 8083 |
