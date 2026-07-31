# Publishing

`Basic.Publish` is the only path for writing messages in AMQP. A single publish consists of three frames: the `Basic.Publish` method frame (carrying the exchange, routing-key, and `mandatory` flag), a Content Header frame (message properties), and one or more Content Body frames (the message body, split according to the negotiated `frame-max`).

## Routing Rules

A published message isn't written directly to a queue — it's first handed to an exchange, which routes it to matching queues (zero, one, or many) based on its type:

- **Default exchange** (`exchange=""`): implicit routing where the routing-key *is* the target queue name, written directly to that queue by name. If the target queue hasn't been explicitly declared via `Queue.Declare` yet, RobustMQ auto-creates it on publish — this accommodates clients that publish before declaring a queue.
- **`direct`**: routes only when the routing-key **exactly equals** a binding's binding-key.
- **`fanout`**: ignores the routing-key, broadcasts to every queue bound to that exchange.
- **`topic`**: the routing-key is split by `.` and matched against the binding pattern (`*` matches one segment, `#` matches zero or more).
- **`headers`**: ignores the routing-key, matches header key/value pairs declared at bind time along with `x-match` (`all`/`any`).
- **Exchange-to-exchange bindings**: if the destination is another exchange (rather than a queue), the message continues routing along the binding chain until it lands on a concrete queue; cyclic bindings are detected and skipped rather than forwarded forever.

## `mandatory` and Unroutable Messages

The `mandatory` flag on `Basic.Publish` determines what happens when a message can't be routed to any queue:

- `mandatory=true`: the broker returns the message to the publisher via `Basic.Return` (`reply-code=312 NO_ROUTE`), immediately followed by the original Content Header + Body, so the client gets the full returned message.
- `mandatory=false` (default): silently dropped, the publisher isn't notified.

The `immediate` flag (requiring the message be delivered to an online consumer immediately, or returned) is widely considered an anti-pattern in AMQP 0-9-1; RobustMQ matches modern RabbitMQ and gives it no special handling.

## Message Properties

The standard properties carried in the Content Header (`content-type`, `content-encoding`, `delivery-mode`, `priority`, `correlation-id`, `reply-to`, `expiration`, `message-id`, `timestamp`, `type`, `user-id`, `app-id`, `cluster-id`, and a custom `headers` table) are fully preserved and carried back unchanged on consumption (`Basic.Get-Ok` / `Basic.Deliver`) as well as on redelivery.

> `delivery-mode=2` (persistent) vs. `delivery-mode=1` (non-persistent) currently has no effect on storage behavior — RobustMQ's storage engine persists uniformly per queue and doesn't distinguish per-message persistence flags.

## Reliable Publishing

`Basic.Publish` itself is asynchronous and waits for no acknowledgement. If you need confirmation that a message was actually written successfully, use [Publisher Confirms](./PublisherConfirms.md) (`Confirm.Select`); if you need to be notified when a message can't be routed, use the `mandatory` flag above.

## Example (Java Client)

```java
Channel channel = connection.createChannel();

// Publish directly to a queue via the default exchange (auto-creates the queue if needed)
channel.basicPublish("", "orders", null, "order-1".getBytes());

// Route through a named direct exchange
channel.exchangeDeclare("orders-exchange", "direct", true);
channel.queueDeclare("orders-queue", true, false, false, null);
channel.queueBind("orders-queue", "orders-exchange", "orders.created");
channel.basicPublish("orders-exchange", "orders.created", null, "order-2".getBytes());

// mandatory: require a return if it can't be routed
AMQP.BasicProperties props = new AMQP.BasicProperties.Builder()
        .contentType("application/json")
        .build();
channel.basicPublish("orders-exchange", "orders.cancelled", true, props, "order-3".getBytes());
```

## Further Reading

- [Core Concepts](./AMQPCoreConcepts.md) — exchange types and routing rules
- [Publisher Confirms](./PublisherConfirms.md) — reliable publishing
- [Consuming](./Consuming.md)
