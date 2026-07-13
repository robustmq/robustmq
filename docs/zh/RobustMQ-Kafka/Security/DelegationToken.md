# 委托令牌

RobustMQ Kafka 实现了委托令牌(Delegation Token,KIP-48)的元数据管理接口:`CreateDelegationToken` / `RenewDelegationToken` / `ExpireDelegationToken` / `DescribeDelegationToken`。

> **现状(务必留意):这是纯元数据管理。令牌的 HMAC 不参与认证** —— Broker(以及其它任何路径)都不会在认证时校验令牌 HMAC。SASL 目前只支持 SCRAM,不存在基于委托令牌的认证机制。因此这些接口用于协议兼容的令牌生命周期管理,但令牌本身当前不能用来登录。

## 行为要点

| 方面 | 说明 |
|---|---|
| 签名密钥 | Broker **自动生成** 32 字节随机密钥,存入 meta-service 资源配置并跨节点共享 |
| HMAC | 以 `HmacSha256(secret, token_id)` 计算,仅作为 Renew/Expire 的查找键与响应回显 |
| 归属 | 请求者固定为占位主体 `User:ANONYMOUS`(无 principal 鉴权);未显式指定 owner 时归属回落到请求者 |
| 校验 | Renew/Expire 仅按提交的 HMAC 定位令牌,**不校验调用者是否为 owner/renewer** |
| 过期回收 | 后台回收器每 60 秒清理超过 `max_timestamp_ms` 的令牌 |

## CLI 示例

在 secure(SASL 开启)模式下,`kafka-delegation-tokens.sh` 需要 `--command-config` 指定 SASL 配置:

```bash
# 创建令牌
kafka-delegation-tokens.sh --bootstrap-server localhost:9092 \
  --command-config client-sasl.properties \
  --create --max-life-time-period -1

# 查询
kafka-delegation-tokens.sh --bootstrap-server localhost:9092 \
  --command-config client-sasl.properties --describe

# 续期 / 过期
kafka-delegation-tokens.sh --bootstrap-server localhost:9092 \
  --command-config client-sasl.properties \
  --renew --renew-time-period -1 --hmac <HMAC>
```

令牌元数据同样经 Raft 持久化并广播到各 Broker 缓存(见 [安全总览](./Overview.md#数据持久化路径))。

## 限制小结

| 限制 | 说明 |
|---|---|
| 不参与认证 | 令牌 HMAC 不用于登录,无 `SASL/SCRAM` 委托令牌重认证流程 |
| 无归属鉴权 | 归属固定 `User:ANONYMOUS`;续期/过期不校验调用者身份 |
| 纯元数据 | 仅提供令牌的创建/续期/过期/查询与自动回收 |
