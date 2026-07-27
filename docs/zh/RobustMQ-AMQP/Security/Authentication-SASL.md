# 认证(SASL)

AMQP 0-9-1 通过 `Connection.Start`/`Connection.StartOk`/`Connection.Secure`/`Connection.Open` 的握手序列完成认证。RobustMQ 目前支持 SASL **PLAIN** 机制。

## 握手流程

1. Broker 发送 `Connection.Start`,通过 `mechanisms` 字段告知客户端自己支持的 SASL 机制——目前只广播 `PLAIN`。
2. 客户端回复 `Connection.StartOk`,其中 `response` 字段按 [RFC 4616](https://www.rfc-editor.org/rfc/rfc4616) 定义的 SASL PLAIN 格式编码:`\0username\0password`。
3. Broker 严格按该格式解析 `response`;格式不符或声明了非 PLAIN 机制的客户端,会被以 `530 NOT_ALLOWED` 关闭连接。
4. 解析出的用户名密码先被缓存,真正的校验发生在客户端发送 `Connection.Open` 时。

## 认证何时真正生效

`Connection.Open` 携带的 `virtual_host` 会作为租户标识,与之前缓存的用户名密码一起,提交给 RobustMQ 统一的用户体系做校验(与 MQTT、Kafka 共用同一套用户数据,不是 AMQP 独立维护的一份)。`virtual_host` 留空时映射到默认租户。

## 密码校验规则

- 如果该用户配置了密码盐(salt),校验方式是 `SHA-256(salt + password)` 与存储值比对。
- 如果没有配置盐,则直接明文比对。

## 不支持的机制

- **AMQPLAIN**(RabbitMQ 自定义的字段表机制)未实现,客户端如果强制使用该机制会认证失败。
- 客户端库(如 RabbitMQ Java Client)默认通常就是 PLAIN,一般不需要额外配置。

## 示例(Java 客户端)

```java
ConnectionFactory factory = new ConnectionFactory();
factory.setHost("localhost");
factory.setPort(5672);
factory.setUsername("app_user");
factory.setPassword("app_password");
factory.setVirtualHost("/");   // 作为租户标识参与登录校验
Connection connection = factory.newConnection();
```

## 延伸阅读

- [安全概览](./Overview.md)
- [Broker 配置](../Configuration/BrokerConfig.md)
- [快速开始](../QuickStart.md)
