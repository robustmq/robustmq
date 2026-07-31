# Roadmap

RobustMQ AMQP already lets standard AMQP 0-9-1 clients (e.g. the RabbitMQ Java Client) run the core publish/consume/acknowledge flow end to end, while a number of capabilities common in the RabbitMQ ecosystem remain unimplemented. This page gives an honest snapshot of where things stand; see [Protocol Support](./Protocol.md) for the per-method breakdown and [Compatibility & Limitations](./Compatibility-and-Limitations.md) for the overall boundary.

## Completed

| Capability | Notes |
|---|---|
| Connections & channels | Lifecycle management, heartbeats, `Channel.Flow` |
| Exchanges | All four types — direct/fanout/topic/headers, plus exchange-to-exchange bindings |
| Queues | Declare (with real passive counts), bind, purge, delete (with if-unused/if-empty) |
| Publish & consume | `Basic.Publish`/`Get`/`Consume`/`Cancel` |
| Acknowledgement | `Basic.Ack`/`Nack`/`Reject`/`Recover`, automatic requeue on disconnect |
| Reliable publishing | Real Publisher Confirms (acked only after persistence) |
| Exclusive consumption | `exclusive` is genuinely enforced |
| Authentication | SASL PLAIN, sharing the unified user store |
| Shared consume groups | Queues reuse MQTT/NATS shared-consume-group infrastructure, supporting competing consumers and cross-node forwarding |

## In Progress / Planned

| Capability | Notes |
|---|---|
| ACL / operation authorization | Currently no permission check after authentication; exchange/queue-level authorization is planned |
| TLS/SSL | The AMQP port currently only accepts plaintext TCP |
| Declare-argument enforcement | Policy arguments like `x-message-ttl`/`x-expires`/`x-max-length`/`x-max-priority` are currently stored but not acted on |
| Dead-letter queues (DLX) | Rejected/discarded messages are currently deleted outright; forwarding to a dead-letter exchange is planned |
| Real `durable`/`auto-delete` semantics | Persistence behavior currently doesn't depend on the `durable` value; differentiating them is planned |
| Cross-node prefetch | `Basic.Qos` across nodes is currently best-effort; strong consistency is planned |
| Transactions (Tx) | Currently only replies with acknowledgement frames; real atomic-commit semantics or explicit deprecation is planned |
| AMQPLAIN | Only SASL PLAIN is currently supported |

## How to Read the Current State

RobustMQ's AMQP implementation follows a "get the core message path solid first, then fill in the management plane" pace: publish, routing, consumption, and acknowledgement are already real semantics, while declare arguments (TTL/DLX/priority, etc.), fine-grained permissions, and transport encryption — the management-plane and edge capabilities — are still being planned. If your use case depends heavily on these, read [Compatibility & Limitations](./Compatibility-and-Limitations.md) first to assess the impact.

## Further Reading

- [Protocol Support](./Protocol.md)
- [Compatibility & Limitations](./Compatibility-and-Limitations.md)
- [System Architecture](./SystemArchitecture.md)
