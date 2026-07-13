# SASL/SCRAM 认证

RobustMQ Kafka 的客户端认证基于 SASL,机制默认支持 **SCRAM-SHA-256** 与 **SCRAM-SHA-512**(遵循 RFC 5802)。开启后,未完成认证的连接只被允许发送握手相关请求,其余请求一律拒绝。

## 握手时序

![SCRAM 握手时序](../../../images/kafka-scram.svg)

1. **SaslHandshake**:客户端选择机制。服务端校验该机制既在配置的可用列表中、又是实现支持的 SCRAM 机制,否则返回 `UNSUPPORTED_SASL_MECHANISM`,并回传可用机制列表。
2. **SaslAuthenticate · client-first**:客户端发送 `n,,n=<user>,r=<cNonce>`(仅接受 `n,,` 这一 gs2 头,不支持通道绑定与 authzid)。若用户不存在,服务端返回与"密码错误"完全相同的通用失败信息,以避免用户枚举。
3. **SaslAuthenticate · server-first**:服务端回传 `r=<cNonce+sNonce>,s=<salt>,i=<iterations>`(服务端随机 nonce)。
4. **SaslAuthenticate · client-final**:客户端发送 `c=biws,r=...,p=<clientProof>`。服务端用存储的 `StoredKey` 校验 proof。
5. **SaslAuthenticate · server-final**:校验通过后回传 `v=<serverSignature>`,连接进入已认证状态,principal 为该用户名。

## 凭据管理

服务端只存储从口令派生出的密钥,**不存储明文口令,也不存储 SaltedPassword**:

| 字段 | 说明 |
|---|---|
| `mechanism` | `1` = SCRAM-SHA-256,`2` = SCRAM-SHA-512 |
| `iterations` | 迭代次数,强制 `>= 4096` |
| `salt` | 随机盐 |
| `stored_key` | `H(HMAC(SaltedPassword, "Client Key"))` |
| `server_key` | `HMAC(SaltedPassword, "Server Key")` |

`DescribeUserScramCredentials` 只返回机制与迭代次数,**永不返回盐或密钥**。

用 kafka-configs 增删凭据(由 `AlterUserScramCredentials` 承载):

```bash
# 新增/更新用户 alice 的 SCRAM-SHA-256 凭据
kafka-configs.sh --bootstrap-server localhost:9092 \
  --alter --entity-type users --entity-name alice \
  --add-config 'SCRAM-SHA-256=[iterations=8192,password=alice-secret]'

# 删除
kafka-configs.sh --bootstrap-server localhost:9092 \
  --alter --entity-type users --entity-name alice \
  --delete-config 'SCRAM-SHA-256'
```

凭据经 meta-service 的 Raft 写入并持久化,随后广播到各 Broker 缓存;Broker 启动时也会全量加载一次(详见 [安全总览 · 数据持久化路径](./Overview.md#数据持久化路径))。

## 客户端配置

```properties
security.protocol=SASL_PLAINTEXT
sasl.mechanism=SCRAM-SHA-256
sasl.jaas.config=org.apache.kafka.common.security.scram.ScramLoginModule required \
  username="alice" password="alice-secret";
```

服务端开启 SASL:

```toml
[kafka.sasl]
enabled = true
mechanisms = ["SCRAM-SHA-256", "SCRAM-SHA-512"]
```

## 限制

| 限制 | 说明 |
|---|---|
| 仅 SCRAM | 不支持 PLAIN / OAUTHBEARER / GSSAPI(Kerberos);且无法通过配置新增其它机制 |
| 无重认证 | KIP-368 重认证暂缓,服务端不设 `session_lifetime`(返回 0,即不强制重认证窗口) |
| 无传输加密 | 当前为 `SASL_PLAINTEXT`,不含 TLS;口令派生数据在网络上受 SCRAM 协议保护,但通道本身未加密 |
| 单租户认证视图 | 凭据在存储层按租户隔离,但 Broker 认证查找走默认租户 |

> ACL 授权尚未强制,认证通过后不会再按 ACL 拦截操作,详见 [ACL 授权](./Authorization-ACL.md)。
