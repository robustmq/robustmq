# Publisher Confirms

Publisher Confirms is AMQP 0-9-1's extension protocol (the `Confirm` class), letting a producer reliably know whether a message has actually been received and persisted by the broker, instead of firing and forgetting. RobustMQ implements real Publisher Confirm semantics.

## Why It's Needed

Plain `Basic.Publish` is fire-and-forget: without Publisher Confirms, a producer can't tell "the message was written successfully" apart from "it was lost on the wire" or "the broker failed to process it." Once a channel enters confirm mode, the broker actively replies with a `Basic.Ack` (or `Basic.Nack` on failure) after the message is actually persisted.

## Confirm.Select

A producer switches a channel into confirm mode with `Confirm.Select`; the broker replies `Confirm.Select-Ok`. After the switch, every `Basic.Publish` on that channel is tracked.

## Publish Sequence Numbers and Confirms

- Once in confirm mode, every `Basic.Publish` on the channel is assigned a **publish sequence number**, starting at 1 and increasing monotonically (a separate counter from the `delivery-tag` used by `Basic.Ack`/`Nack` on the consume side — one tracks publishing, the other consumption).
- Once a message has been routed and successfully written to storage (File Segment), the broker replies `Basic.Ack` carrying that sequence number.
- `multiple=true` acknowledges every message with a sequence number less than or equal to this one (batched acking, fewer round trips).
- If a message fails to route (no matching queue) or fails to persist, the broker replies `Basic.Nack`.

## Interaction with `mandatory`

`mandatory=true` detects the case where "a message didn't match any queue" (see [Publishing](./Publishing.md)). In confirm mode, an unroutable `mandatory` message first triggers a `Basic.Return`, and is still followed by the corresponding `Basic.Ack`/`Nack` (RobustMQ currently treats "not routed to any queue" as a successful publish, as long as the exchange-matching logic itself didn't error — consistent with how some other broker implementations handle this).

## Example (Java Client)

```java
channel.confirmSelect();
channel.basicPublish("orders-exchange", "order.created", null, payload);

// Synchronously wait for this one confirm (not recommended at high throughput, shown for illustration only)
if (!channel.waitForConfirms(5000)) {
    // handle failure — retry or alert
}
```

For higher throughput, listen asynchronously instead:

```java
channel.confirmSelect();
channel.addConfirmListener(
    (deliveryTag, multiple) -> { /* confirmed */ },
    (deliveryTag, multiple) -> { /* failed — resend or log */ }
);
```

## Limitations

- Publisher Confirms only guarantee that a message was successfully written to storage on the handling node — they don't cover cross-node replica acknowledgement (RobustMQ's high availability comes from the File Segment storage engine and the Raft metadata layer, not per-message multi-replica synchronous writes).
- Transactions (`Tx.Select`/`Tx.Commit`) currently **do not** provide real atomic-commit semantics — they only reply with acknowledgement frames. Don't rely on them for reliability guarantees; use Publisher Confirms instead.

## Further Reading

- [Publishing](./Publishing.md)
- [Acknowledgement](./Acknowledgement.md) — the consume-side counterpart of reliability
- [Protocol Support](./Protocol.md)
