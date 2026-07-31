# Exchange 与 Queue 声明参数

`Exchange.Declare` 和 `Queue.Declare` 除了名字/类型之外还带一堆声明参数,这里说明 RobustMQ 对每个字段的真实处理方式。

## Exchange.Declare 参数

| 参数 | 是否生效 | 说明 |
|---|---|---|
| `type` | ✅ | 决定路由算法,支持 direct/fanout/topic/headers |
| `passive` | ✅ | 只检查是否存在,不创建;不存在返回 `404 NOT_FOUND` |
| `durable` | 🟡 | 会被保存,但目前与 `durable=false` 行为一致(见下文说明) |
| `auto-delete` | 🟡 | 会被保存,但目前不触发"最后一个绑定解除后自动删除" |
| `internal` | 🟡 | 会被保存,但目前不禁止客户端直接向 internal 交换机发布 |
| `arguments` | ❌ | 会被完整保存为键值表,但**不解析、不生效**——例如 `alternate-exchange` 这样的常见参数目前不会被读取执行 |

## Queue.Declare 参数

| 参数 | 是否生效 | 说明 |
|---|---|---|
| `passive` | ✅ | 只检查是否存在,返回**真实的** `message_count`/`consumer_count`;不存在返回 `404 NOT_FOUND` |
| `durable` | 🟡 | 会被保存,但目前与 `durable=false` 行为一致 |
| `exclusive` | ✅ | 声明为独占队列后,其他连接无法访问 |
| `auto-delete` | 🟡 | 会被保存,但目前不触发"最后一个消费者断开后自动删除" |
| `arguments` | ❌ | 会被完整保存为键值表,但**不解析、不生效** |

## 关于 `arguments` 里的常见策略参数

RabbitMQ 客户端常用的一些策略型参数,在 RobustMQ 里目前**全部不生效**,仅作为普通键值对存储在元数据里:

- `x-message-ttl` —— 消息级 TTL,无效
- `x-expires` —— 队列级 TTL,无效
- `x-max-length` / `x-max-length-bytes` —— 队列容量上限,无效
- `x-dead-letter-exchange` / `x-dead-letter-routing-key` —— 死信路由,无效(RobustMQ 没有死信队列实现)
- `x-max-priority` —— 优先级队列,无效

如果客户端代码里声明了这些参数,声明本身不会报错(参数会被静默接受并保存),但对应的策略行为不会发生。规划迁移时请不要依赖这些参数生效。

## 关于 `durable`

无论 `durable` 设置为 `true` 还是 `false`,Exchange/Queue 的元数据都会被同样地持久化写入,重启 Broker 不会清除非持久化(`durable=false`)声明的对象——这与 RabbitMQ 的行为不同(RabbitMQ 里非持久化的对象重启后会消失)。如果你的应用逻辑依赖"非持久化队列在重启后自动消失"这一行为,目前在 RobustMQ 上不成立。

## 延伸阅读

- [核心概念](../AMQPCoreConcepts.md)
- [存储](../Storage.md)
- [兼容性与限制](../Compatibility-and-Limitations.md)
