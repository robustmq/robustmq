# Authentication (SASL)

AMQP 0-9-1 authenticates through the `Connection.Start`/`Connection.StartOk`/`Connection.Secure`/`Connection.Open` handshake sequence. RobustMQ currently supports the SASL **PLAIN** mechanism.

## Handshake Flow

1. The broker sends `Connection.Start`, advertising its supported SASL mechanisms via the `mechanisms` field — today it only advertises `PLAIN`.
2. The client replies with `Connection.StartOk`, whose `response` field is encoded per [RFC 4616](https://www.rfc-editor.org/rfc/rfc4616)'s SASL PLAIN format: `\0username\0password`.
3. The broker strictly parses `response` in that format; a malformed response, or a client declaring a mechanism other than PLAIN, gets the connection closed with `530 NOT_ALLOWED`.
4. The parsed username/password is cached first; the actual verification happens when the client sends `Connection.Open`.

## When Authentication Actually Takes Effect

The `virtual_host` carried in `Connection.Open` is used as a tenant identifier, together with the previously cached username/password, and submitted to RobustMQ's unified user store for verification (shared with MQTT and Kafka's user data, not a separate store maintained just for AMQP). An empty `virtual_host` maps to the default tenant.

## Password Verification Rules

- If the user has a configured password salt, verification compares `SHA-256(salt + password)` against the stored value.
- If no salt is configured, it's a direct plaintext comparison.

## Unsupported Mechanisms

- **AMQPLAIN** (RabbitMQ's custom field-table mechanism) is not implemented; a client forced to use it will fail to authenticate.
- Client libraries (e.g. the RabbitMQ Java Client) typically default to PLAIN already, so no extra configuration is usually needed.

## Example (Java Client)

```java
ConnectionFactory factory = new ConnectionFactory();
factory.setHost("localhost");
factory.setPort(5672);
factory.setUsername("app_user");
factory.setPassword("app_password");
factory.setVirtualHost("/");   // used as the tenant identifier during login
Connection connection = factory.newConnection();
```

## Further Reading

- [Security Overview](./Overview.md)
- [Broker Configuration](../Configuration/BrokerConfig.md)
- [Quick Start](../QuickStart.md)
