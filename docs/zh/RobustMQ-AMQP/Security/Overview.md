# 安全概览

RobustMQ AMQP 目前只实现了**身份认证(Authentication)**,尚未实现**授权(Authorization/ACL)**,也不支持 TLS。规划安全策略时请注意这个边界。

## 当前能力

| 能力 | 状态 | 说明 |
|---|---|---|
| SASL 认证 | ✅ | 仅支持 `PLAIN` 机制,详见 [认证(SASL)](./Authentication-SASL.md) |
| 用户名密码校验 | ✅ | 复用 RobustMQ 统一的用户体系,与 MQTT/Kafka 共享同一份用户数据 |
| vhost 作为租户 | 🟡 | `virtual_host` 被当作租户标识参与登录校验,但没有基于 vhost 的资源隔离或权限控制 |
| ACL / 操作授权 | ❌ | 登录成功后,对 exchange/queue 的声明、绑定、发布、消费等操作没有权限校验 |
| TLS/SSL | ❌ | AMQP 端口(默认 5672)只支持明文 TCP,没有加密传输选项 |

## 认证边界要清楚

登录成功只意味着"用户名密码正确、这个连接被接受",不意味着这个用户对任何具体的 exchange 或 queue 有细粒度的操作权限——因为这部分授权检查目前没有实现。也就是说,任何认证通过的客户端都可以声明、绑定、发布、消费集群内的任意队列和交换机。在生产环境中使用时,需要通过网络层面(如安全组、防火墙、只对可信客户端开放端口)来弥补这个缺口。

## 传输安全

由于没有 TLS,AMQP 流量(包括 SASL PLAIN 认证时携带的明文用户名密码)在网络上是不加密的。如果需要加密传输,目前只能通过外部手段(如在可信内网部署、或在客户端与 Broker 之间加一层 stunnel/service mesh mTLS)实现,RobustMQ 本身尚未内建 TLS 终止。

## 延伸阅读

- [认证(SASL)](./Authentication-SASL.md)
- [兼容性与限制](../Compatibility-and-Limitations.md)
- [Broker 配置](../Configuration/BrokerConfig.md)
