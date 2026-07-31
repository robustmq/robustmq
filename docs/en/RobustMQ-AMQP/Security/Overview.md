# Security Overview

RobustMQ AMQP currently implements only **authentication**, not **authorization/ACL**, and does not support TLS. Keep this boundary in mind when planning a security posture.

## Current Capabilities

| Capability | Status | Notes |
|---|---|---|
| SASL authentication | ✅ | Only the `PLAIN` mechanism is supported — see [Authentication (SASL)](./Authentication-SASL.md) |
| Username/password verification | ✅ | Reuses RobustMQ's unified user store, shared with MQTT/Kafka |
| vhost as tenant | 🟡 | `virtual_host` is used as a tenant identifier during login, but there's no vhost-scoped resource isolation or permission control beyond that |
| ACL / operation authorization | ❌ | Once logged in, there's no permission check on declaring/binding exchanges or queues, publishing, or consuming |
| TLS/SSL | ❌ | The AMQP port (default 5672) only accepts plaintext TCP — no encrypted transport option |

## Know the Authentication Boundary

A successful login only means "the username/password was correct and this connection is accepted" — it does NOT mean this user has fine-grained permission on any specific exchange or queue, because that authorization layer isn't implemented yet. In practice, any authenticated client can declare, bind, publish to, and consume from any queue or exchange in the cluster. If you're running this in production, you need to close this gap at the network layer instead (security groups, firewalls, only exposing the port to trusted clients).

## Transport Security

Without TLS, AMQP traffic — including the plaintext username/password carried in SASL PLAIN authentication — is unencrypted on the wire. If you need encrypted transport today, the only options are external: deploy within a trusted private network, or add a layer like stunnel or a service-mesh mTLS sidecar between clients and the broker. RobustMQ itself doesn't yet have built-in TLS termination.

## Further Reading

- [Authentication (SASL)](./Authentication-SASL.md)
- [Compatibility & Limitations](../Compatibility-and-Limitations.md)
- [Broker Configuration](../Configuration/BrokerConfig.md)
