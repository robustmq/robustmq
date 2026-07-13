# ACL 授权

RobustMQ Kafka 实现了 Kafka ACL 的管理接口(`CreateAcls` / `DescribeAcls` / `DeleteAcls`),可对 ACL 规则进行增删查并持久化到 meta-service。

> **现状(务必留意):ACL 可增删查并持久化,但尚未接入请求鉴权强制。** 请求处理路径不会读取 ACL,因此当前**不会真正拦截**任何 Produce / Fetch / 元数据等操作。`Metadata` 等响应中的 `authorized_operations` 字段一律返回哨兵值(未计算)。ACL 目前是"可管理的授权元数据",可用于提前配置、迁移演练,但不构成访问控制。

## ACL 模型

Kafka 线上模型字段如下:

| 字段 | 说明 |
|---|---|
| `resource_type` | 资源类型。**当前仅接受 `TOPIC`** |
| `resource_name` | 资源名(topic 名) |
| `pattern_type` | 匹配模式。**当前仅接受 `LITERAL`(精确)** |
| `principal` | 主体,形如 `User:alice` 或 `ClientId:x`(缺少 `:` 报 `InvalidPrincipalType`) |
| `host` | 来源主机;空值归一为 `*` |
| `operation` | 操作。映射为内部动作:`ALL`→全部、`READ`→订阅、`WRITE`→发布;其余报 `InvalidRequest` |
| `permission_type` | `ALLOW` / `DENY` |

非 `TOPIC` 资源、或非 `LITERAL` 模式,均返回 `InvalidRequest`。

## 过滤匹配

`DescribeAcls` / `DeleteAcls` 通过过滤器匹配:枚举类字段设为 `ANY` 时匹配任意值;字符串过滤(资源名、主体、主机)为空时匹配任意,否则做**精确字符串相等**比较。前缀/通配(`PREFIXED` / `MATCH`)的真正前缀匹配尚未实现。

## CLI 示例

```bash
# 授予 alice 对 topic orders 的读权限
kafka-acls.sh --bootstrap-server localhost:9092 \
  --add --allow-principal User:alice \
  --operation Read --topic orders

# 查询
kafka-acls.sh --bootstrap-server localhost:9092 --list --topic orders

# 删除
kafka-acls.sh --bootstrap-server localhost:9092 \
  --remove --allow-principal User:alice \
  --operation Read --topic orders
```

ACL 与其它安全数据一样,经 Raft 持久化并广播到各 Broker 缓存(见 [安全总览](./Overview.md#数据持久化路径))。

## 限制小结

| 限制 | 说明 |
|---|---|
| 未强制 | 不参与请求鉴权,不会拦截操作 |
| 资源类型 | 仅 `TOPIC` |
| 模式类型 | 仅 `LITERAL`,无前缀/通配匹配 |
| 操作类型 | 仅 `ALL` / `READ` / `WRITE` 可映射 |
| authorized_operations | 响应中恒为"未计算"哨兵值 |
