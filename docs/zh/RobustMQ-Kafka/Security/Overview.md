# 安全总览

RobustMQ Kafka 在协议层实现了 Kafka 的安全相关 API,并把认证凭据、ACL、配额、委托令牌等安全数据统一持久化在基于 Raft 的 meta-service 中。本页概述当前的安全能力边界,并指向各专题文档。

> 需要特别诚实地说明:目前**只有 SASL/SCRAM 认证是端到端强制生效的**。ACL、配额、委托令牌都已实现了完整的协议兼容管理接口(可增删查、持久化、集群广播),但**尚未接入请求处理路径进行强制**。也就是说,它们现在是"可管理的元数据",而不是"会真正拦截/限流/鉴权"的机制。

## 能力矩阵

| 能力 | 相关 API | 状态 | 说明 |
|---|---|---|---|
| SASL/SCRAM 认证 | SaslHandshake / SaslAuthenticate / AlterUserScramCredentials / DescribeUserScramCredentials | ✅ 强制生效 | 仅 SCRAM-SHA-256 / SCRAM-SHA-512;开启后未认证连接只能发握手类请求 |
| ACL 授权 | CreateAcls / DescribeAcls / DeleteAcls | 🟡 可管理,未强制 | 可增删查并持久化,但请求处理路径不读取 ACL,不会真正拦截 |
| 客户端配额 | AlterClientQuotas / DescribeClientQuotas | 🟡 可管理,未强制 | 仅 `client-id` 实体;设置后不会真正限流 |
| 委托令牌 | Create / Renew / Expire / DescribeDelegationToken | 🟡 元数据管理 | HMAC 不参与认证,归属固定为 `User:ANONYMOUS` |

## 数据持久化路径

所有安全数据都走同一条链路:

1. Broker 收到管理请求(如 `AlterUserScramCredentials`),校验后经 gRPC 转发给 meta-service;
2. meta-service 通过 **Raft** 写入 RocksDB(强一致);
3. 写入成功后向各 Broker **广播缓存更新**通知;
4. 各 Broker 在**启动时**从 meta-service 全量加载一次,运行期靠广播增量更新。

因此凭据、ACL、配额在集群内是强一致且各节点视图一致的。

## 默认状态

SASL 默认**关闭**(`kafka.sasl.enabled = false`)。关闭时所有连接均视为已认证、不做握手校验;开启后默认启用的机制列表为 `["SCRAM-SHA-256", "SCRAM-SHA-512"]`。

## 专题文档

- [SASL/SCRAM 认证](./Authentication-SASL-SCRAM.md)
- [ACL 授权](./Authorization-ACL.md)
- [客户端配额](./Quota.md)
- [委托令牌](./DelegationToken.md)

> 存储与协调机制见 [系统架构](../SystemArchitecture.md) 与 [存储引擎](../Storage.md)。
