# 兼容性与限制

本文汇总 RobustMQ AMQP 实现相对于标准 AMQP 0-9-1(以 RabbitMQ 为参照)的能力边界,便于评估迁移可行性。更细粒度的逐方法状态请见 [协议支持](./Protocol.md)。

## 已完整支持

- Connection/Channel 生命周期管理(开启、关闭、心跳、`Channel.Flow`)。
- 四种交换机类型:direct、fanout、topic、headers,以及交换机到交换机的绑定链路。
- Queue 声明(含 `passive`,返回真实 `message_count`/`consumer_count`)、绑定、清空(`Purge`)、删除(`Delete`,含 `if-unused`/`if-empty` 校验)。
- `Basic.Publish`/`Basic.Get`/`Basic.Consume`/`Basic.Cancel`。
- `Basic.Ack`/`Nack`/`Reject`/`Recover`/`Recover-Async`,断线自动 requeue。
- `Confirm.Select` 及真实的 Publisher Confirm(消息落盘后才 ack)。
- 独占消费(`exclusive`)强制生效。
- SASL PLAIN 认证,复用 RobustMQ 统一用户体系。

## 部分支持 / 有限制

| 特性 | 限制说明 |
|---|---|
| `Basic.Qos` prefetch | 仅在消费者与队列 leader 同节点时强制生效,跨节点为尽力而为;`prefetch-size`(按字节)不生效 |
| `durable` / `auto-delete` | 参数会被保存,但目前不影响实际持久化或自动删除行为(durable/非 durable 表现一致) |
| `no-local` | 声明后不过滤本连接自己发布的消息 |
| `Tx.*`(事务) | 只回复确认帧,不提供真实原子提交,不要用于可靠性保证,请用 Publisher Confirm 代替 |
| `internal` 交换机 | 会被保存,但不阻止客户端直接发布到 internal 交换机 |

## 未实现

- **ACL / 操作授权**:认证通过后,对 exchange/queue 的所有操作没有权限校验。
- **TLS/SSL**:AMQP 端口只支持明文 TCP。
- **AMQPLAIN** SASL 机制:只支持 PLAIN。
- **死信队列(DLX)**:被拒绝/丢弃的消息直接删除,不转发。
- **消息/队列 TTL**:`x-message-ttl`、`x-expires` 等参数不生效。
- **队列容量上限**:`x-max-length`、`x-max-length-bytes` 不生效。
- **优先级队列**:`x-max-priority` 不生效。
- **惰性队列(Lazy Queue)**:所有消息统一走 File Segment 存储,无内存/磁盘两级区分。
- **Federation / Shovel / 集群镜像队列**等 RabbitMQ 生态插件特性:均未实现,RobustMQ 的高可用通过 Raft 元数据层与 File Segment 存储引擎本身提供,机制不同,不能按 RabbitMQ 插件配置方式迁移。

## 迁移建议

如果你正在从 RabbitMQ 迁移到 RobustMQ:

1. 检查你的应用是否依赖 `arguments` 里的策略型参数(TTL、DLX、优先级、容量上限)——目前都需要在应用层自行实现或改造。
2. 检查是否依赖细粒度权限控制(vhost 级或 exchange/queue 级 ACL)——目前需要通过网络层隔离弥补。
3. 检查是否依赖非持久化对象在重启后消失的行为——目前不成立。
4. 如果消费者分布在多个节点且强依赖精确的 prefetch 限流,注意跨节点场景目前是尽力而为。

## 延伸阅读

- [协议支持](./Protocol.md)
- [安全概览](./Security/Overview.md)
- [Exchange 与 Queue 声明参数](./Configuration/ExchangeAndQueueConfig.md)
- [路线图](./Roadmap.md)
