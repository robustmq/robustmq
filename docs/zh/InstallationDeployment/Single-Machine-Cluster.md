# 单机运行 RobustMQ

本指南介绍如何在单台机器上运行 RobustMQ，适用于开发测试环境。

## 准备工作

### 下载二进制包

```bash
# 下载最新版本
wget https://github.com/robustmq/robustmq/releases/download/v1.0.0/robustmq-v1.0.0-linux-amd64.tar.gz

# 解压
tar -xzf robustmq-v1.0.0-linux-amd64.tar.gz
cd robustmq-v1.0.0-linux-amd64
```

## 启动 RobustMQ

### 使用默认配置启动

```bash
# 启动 RobustMQ
./bin/broker-server start
```

### 使用配置文件启动

```bash
# 使用配置文件启动
./bin/broker-server start config/server.toml
```

### 后台启动

```bash
# 后台启动
nohup ./bin/broker-server start > robustmq.log 2>&1 &
```

## 验证运行状态

### 查看服务状态

```bash
# 检查 MQTT 端口
netstat -tlnp | grep 1883

# 检查管理端口
netstat -tlnp | grep 8080
```

### 测试 MQTT 连接

```bash
# 发送消息
mqttx pub -h 127.0.0.1 -p 1883 -t "test/topic" -m "Hello RobustMQ!"

# 订阅消息
mqttx sub -h 127.0.0.1 -p 1883 -t "test/topic"
```

## 停止服务

```bash
# 停止 RobustMQ
pkill -f "broker-server"

# 或者查找进程 ID 后停止
ps aux | grep broker-server
kill <PID>
```

## 默认端口

| 服务 | 端口 | 描述 |
|------|------|------|
| MQTT | 1883 | MQTT 协议端口 |
| Admin | 8080 | 管理接口端口 |
| Cluster | 9090 | 集群通信端口 |
| Meta | 9091 | 元数据服务端口 |

## 注意事项

1. **端口占用**: 确保默认端口未被占用
2. **防火墙**: 确保防火墙允许相关端口通信
3. **资源要求**: 建议至少 2GB 内存
4. **数据目录**: RobustMQ 会在当前目录创建数据文件
