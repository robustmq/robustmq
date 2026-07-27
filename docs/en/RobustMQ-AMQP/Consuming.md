# Consuming

AMQP 0-9-1 offers two complementary ways to consume: `Basic.Get` (a synchronous pull of one message) and `Basic.Consume` (register a consumer that the broker pushes to). Both share the same underlying read cursor — `Get`-ing then `Consume`-ing the same queue (or vice versa) neither double-consumes nor skips messages.

## Pull Consumption: `Basic.Get`

`Basic.Get` is a one-shot, synchronous operation: send a request, get back one message (or `Basic.Get-Empty` if the queue is empty).

- If the queue's current [shared consume group leader](./SharedQueueGroup.md) isn't the node handling the request, RobustMQ forwards the `Get` to the leader node via one internal gRPC call, fully transparent to the client.
- With `no-ack=true`, a message is considered consumed as soon as it's read and never enters the unacked state; with `no-ack=false` (default), it enters the unacked state and requires an explicit `Basic.Ack`/`Nack`/`Reject`.

`Basic.Get` suits low-frequency polling or one-off tasks — for high throughput, use `Basic.Consume` instead.

## Push Consumption: `Basic.Consume`

Once a consumer is registered via `Basic.Consume`, the broker continuously pushes messages to it via `Basic.Deliver` until it's cancelled with `Basic.Cancel` or the connection/channel closes.

- Multiple consumers (multiple channels on the same connection, or different connections from different nodes) calling `Basic.Consume` on the same queue become members of that queue's [shared consume group](./SharedQueueGroup.md), **competing** with each other — each message goes to only one of them.
- The push task on the queue's leader node handles actual delivery; if a consumer's node isn't the leader, the leader delivers via an internal gRPC call (`SendShareGroupMessage`) to whichever connection the consumer lives on.
- If `consumer-tag` is left blank, the broker generates a unique one, avoiding collisions when multiple consumers on the same connection all leave it blank.
- `no-local`: declaring it has no effect — messages published by the same connection are not filtered out (matching RabbitMQ classic queue behavior, where this flag was never enforced for classic queues either).
- `exclusive=true`: requests sole ownership of the queue. If the queue already has other consumers (or an existing exclusive consumer), registration is rejected (`403 ACCESS_REFUSED`).

## Prefetch and `Basic.Qos`

`Basic.Qos`'s `prefetch-count` limits how many unacked messages a consumer can hold at once, protecting it from being overwhelmed.

- When a consumer's node **is** the queue's shared consume group leader, prefetch is strictly enforced: once the limit is hit, the push task skips that consumer until it acks some messages and frees up quota.
- When a consumer's node **isn't** the leader, prefetch is currently best-effort — real-time cross-node prefetch-quota synchronization isn't implemented yet; see [Compatibility & Limitations](./Compatibility-and-Limitations.md).
- `prefetch-size` (byte-based limiting) has no effect; the `global` flag is currently always treated at channel granularity.

## Delivery Tags

Each channel maintains its own independent delivery-tag counter, starting at 1 and increasing monotonically. `Basic.Get-Ok` and `Basic.Deliver` share the same counter sequence, never repeating for the channel's lifetime, so `Basic.Ack`/`Nack`/`Reject` can address a specific message later.

## Example (Java Client)

```java
// Pull
GetResponse resp = channel.basicGet("orders-queue", false);
if (resp != null) {
    System.out.println(new String(resp.getBody()));
    channel.basicAck(resp.getEnvelope().getDeliveryTag(), false);
}

// Push: prefetch=1, manual ack
channel.basicQos(1);
channel.basicConsume("orders-queue", false, (consumerTag, delivery) -> {
    System.out.println(new String(delivery.getBody()));
    channel.basicAck(delivery.getEnvelope().getDeliveryTag(), false);
}, consumerTag -> {});
```

## Further Reading

- [Shared Queue Group](./SharedQueueGroup.md) — competing consumers and leader election
- [Acknowledgement](./Acknowledgement.md) — ack / nack / reject / recover
- [Core Concepts](./AMQPCoreConcepts.md)
