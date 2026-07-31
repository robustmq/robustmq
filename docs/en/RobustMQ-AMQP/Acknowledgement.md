# Acknowledgement

Once a message has been delivered (via `Basic.Get-Ok` or `Basic.Deliver`) and the consumer isn't in `no-ack` mode, it enters the **unacked** state and must be explicitly acknowledged or rejected — otherwise it's treated as an unresolved delivery and eventually redelivered.

## The Unacked Index

RobustMQ keeps an index entry for every delivered-but-unacked message, recording which connection and channel it was delivered to and its delivery-tag. This index is what lets `Basic.Ack`/`Nack`/`Reject`/`Recover` locate a specific message, and what lets a connection/channel that closes mid-flight requeue everything it was still holding.

## Basic.Ack

Confirms that one (or more) messages have been fully processed; the broker then deletes them from storage:

- `delivery-tag` identifies the message to acknowledge.
- `multiple=true` acknowledges **every unacked message with a delivery-tag less than or equal to this one** — useful for batch processing, where you only need to ack the last message in a batch.

## Basic.Nack / Basic.Reject

Semantically identical — `Nack` is RabbitMQ's extension to the standard `Reject`, adding `multiple` batching:

- `requeue=true`: the message is put back on the queue for redelivery (possibly to the same consumer, possibly to another group member, depending on round-robin order). On redelivery the `redelivered` flag is set.
- `requeue=false`: the message is discarded outright. RobustMQ currently has **no dead-letter queue** — discarding means permanent deletion, not forwarding anywhere.

## Basic.Recover / Basic.Recover-Async

Requests that the broker redeliver **all** unacked messages on this channel, without needing to name individual delivery-tags. The only difference between the two is that `Recover` replies with a `Recover-Ok` confirmation frame and `Recover-Async` does not.

## Automatic Requeue on Connection/Channel Close

If a connection or channel disconnects while it's still holding unacked messages (client crash, network drop, or a normal close), every unacked message on that connection/channel is automatically put back on the queue — nothing is lost. This is why explicit acking with `no-ack=false` is the safer consumption mode: only messages that were actually processed get deleted.

## Example (Java Client)

```java
channel.basicConsume("orders-queue", false, (consumerTag, delivery) -> {
    long tag = delivery.getEnvelope().getDeliveryTag();
    try {
        process(delivery.getBody());
        channel.basicAck(tag, false);
    } catch (RetryableException e) {
        channel.basicNack(tag, false, true);   // requeue
    } catch (Exception e) {
        channel.basicNack(tag, false, false);  // discard
    }
}, consumerTag -> {});
```

## Further Reading

- [Consuming](./Consuming.md)
- [Core Concepts](./AMQPCoreConcepts.md)
- [Publisher Confirms](./PublisherConfirms.md) — the publish-side counterpart of reliability
