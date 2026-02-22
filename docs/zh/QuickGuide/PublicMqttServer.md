# RobustMQ 公共 MQTT 服务器

本指南介绍如何使用 RobustMQ 提供的公共 MQTT 服务器进行测试和开发。

## 服务器信息

### 接入点

| 协议 | 地址 | 端口 | 描述 |
|------|------|------|------|
| MQTT TCP | 117.72.92.117 | 1883 | 标准 MQTT 连接 |
| MQTT SSL/TLS | 117.72.92.117 | 1885 | 加密 MQTT 连接 |
| MQTT WebSocket | 117.72.92.117 | 8083 | WebSocket 连接 |
| MQTT WebSocket SSL | 117.72.92.117 | 8085 | 加密 WebSocket 连接 |
| MQTT QUIC | 117.72.92.117 | 9083 | QUIC 协议连接 |

### 认证信息

- **用户名**: `admin`
- **密码**: `robustmq`

### 管理界面

- **Dashboard**: <http://demo.robustmq.com/>

<div align="center">
  <img src="../../images/web-ui.jpg" width="600"/>
</div>

## 快速体验

> MQTTX CLI 安装参考：[https://mqttx.app/zh/docs/cli](https://mqttx.app/zh/docs/cli)
>
> 也可直接使用 MQTTX Web 客户端：[https://mqttx.app/web-client](https://mqttx.app/web-client#/recent_connections)

### 发布消息

```bash
# 发送消息
mqttx pub -h 117.72.92.117 -p 1883 -u admin -P robustmq -t "test/topic" -m "Hello RobustMQ!"

# 发送 QoS 1 消息
mqttx pub -h 117.72.92.117 -p 1883 -u admin -P robustmq -t "test/qos1" -m "msg" -q 1

# 发送保留消息
mqttx pub -h 117.72.92.117 -p 1883 -u admin -P robustmq -t "test/retained" -m "retained msg" -r
```

### 订阅消息

```bash
# 订阅主题
mqttx sub -h 117.72.92.117 -p 1883 -u admin -P robustmq -t "test/topic"

# 通配符订阅
mqttx sub -h 117.72.92.117 -p 1883 -u admin -P robustmq -t "test/#"
```

### 使用 MQTTX GUI 客户端

连接配置：**Host** `117.72.92.117` · **Port** `1883` · **Username** `admin` · **Password** `robustmq`

<div align="center">
  <img src="../../images/mqttx01.png" width="600"/>
</div>

<div align="center">
  <img src="../../images/mqttx-2.png" width="600"/>
</div>

## 注意事项

1. 这是用于测试的公共服务器，请勿用于生产环境
2. 请勿在消息中传输敏感信息
3. 请合理使用，避免过度占用资源
