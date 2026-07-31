# RobustMQ AMQP Protocol Support

This document lists RobustMQ's **actual support status**, per AMQP 0-9-1 Class/Method, as an AMQP broker. Status uses three levels:

- ✅ **Fully supported**: the protocol semantics are correctly implemented.
- 🟡 **Partially supported**: the handshake/interface works, but the behavior is a simplified implementation or has a known limitation — see the note.
- ❌ **Not supported**: the method itself errors out or has no real semantics.

References:
- [AMQP 0-9-1 Protocol Specification](https://www.rabbitmq.com/resources/specs/amqp0-9-1.pdf)
- [AMQP 0-9-1 XML Definition](https://www.rabbitmq.com/resources/specs/amqp0-9-1.xml)

See [Compatibility & Limitations](./Compatibility-and-Limitations.md) for the reasoning and impact behind each item.

---

## Protocol Basics

AMQP 0-9-1 runs over TCP and transmits data as frames. Each frame consists of a type, channel number, length, payload, and a frame-end marker (`0xCE`).

- Connection-related
![img](../../images/amqp-01.jpg)

- Publish/consume-related

![img](../../images/amqp-02.jpg)

- Broker-internal logic
![img](../../images/amqp-03.jpg)

### Frame Types

| Frame type | ID | Description |
|--------|------|------|
| Method Frame | 1 | Control commands (all Class/Method) |
| Content Header Frame | 2 | Message properties (content-type, delivery-mode, headers, etc.) |
| Content Body Frame | 3 | Message body payload (may be split across frames) |
| Heartbeat Frame | 8 | Keepalive |

Both publishing and delivery are completed via the **Method + Content Header + Content Body** frame combination.

---

## 1. Connection Class

| Class.Method | ID | Direction | Description | Status |
|--------------|------|------|------|------|
| connection.start | 10.10 | S→C | Broker initiates the handshake, advertising supported SASL mechanisms and locale | ✅ |
| connection.start-ok | 10.11 | C→S | Client selects a SASL mechanism and sends the auth response | ✅ PLAIN only |
| connection.secure | 10.20 | S→C | Broker sends a SASL challenge (multi-round auth) | 🟡 Unused (not needed for PLAIN) |
| connection.secure-ok | 10.21 | C→S | Client responds to the SASL challenge | 🟡 Ack-only stub |
| connection.tune | 10.30 | S→C | Broker proposes channel-max, frame-max, heartbeat | ✅ |
| connection.tune-ok | 10.31 | C→S | Client confirms connection parameters | ✅ Server takes the smaller of the client's negotiated values and its own proposal |
| connection.open | 10.40 | C→S | Client opens a virtual host (mapped to a tenant) | ✅ |
| connection.open-ok | 10.41 | S→C | Broker confirms the vhost connection succeeded | ✅ |
| connection.close | 10.50 | Both | Either side initiates connection close (carries an error code) | ✅ |
| connection.close-ok | 10.51 | Both | Confirms close | ✅ |
| connection.blocked | 10.60 | S→C | Connection-level flow-control warning | 🟡 Unused (no active throttling) |
| connection.unblocked | 10.61 | S→C | Clears the flow-control warning | 🟡 Unused |
| connection.update-secret | 10.70 | C→S | Updates auth credentials on an established connection | 🟡 Ack-only stub, no real credential rotation |
| connection.update-secret-ok | 10.71 | S→C | Confirms the update | 🟡 |

---

## 2. Channel Class

| Class.Method | ID | Direction | Description | Status |
|--------------|------|------|------|------|
| channel.open | 20.10 | C→S | Client opens a channel | ✅ |
| channel.open-ok | 20.11 | S→C | Broker confirms the channel is open | ✅ |
| channel.flow | 20.20 | Both | Pause/resume the message flow (backpressure) | 🟡 Only takes effect for push delivery on the local node — see [Compatibility & Limitations](./Compatibility-and-Limitations.md) |
| channel.flow-ok | 20.21 | Both | Confirms the flow command | ✅ |
| channel.close | 20.40 | Both | Closes the channel (carries an error code) | ✅ |
| channel.close-ok | 20.41 | Both | Confirms close | ✅ |

---

## 3. Exchange Class

Exchanges are the core of message routing, with all four standard types supported: `direct`, `fanout`, `topic`, `headers`.

| Class.Method | ID | Direction | Description | Status |
|--------------|------|------|------|------|
| exchange.declare | 40.10 | C→S | Create or verify an exchange (type/passive/durable/no-wait) | ✅ Including passive semantics (404 if it doesn't exist) |
| exchange.declare-ok | 40.11 | S→C | Confirms creation | ✅ |
| exchange.delete | 40.20 | C→S | Delete an exchange (if-unused option) | ✅ |
| exchange.delete-ok | 40.21 | S→C | Confirms deletion | ✅ |
| exchange.bind | 40.30 | C→S | Exchange-to-exchange binding (RabbitMQ extension) | ✅ Supports chained routing, with cycle protection |
| exchange.bind-ok | 40.31 | S→C | Confirms the binding | ✅ |
| exchange.unbind | 40.40 | C→S | Remove an exchange-to-exchange binding | ✅ |
| exchange.unbind-ok | 40.51 | S→C | Confirms the unbind | ✅ |

---

## 4. Queue Class

| Class.Method | ID | Direction | Description | Status |
|--------------|------|------|------|------|
| queue.declare | 50.10 | C→S | Create or verify a queue (passive/durable/exclusive/auto-delete) | ✅ Including passive semantics |
| queue.declare-ok | 50.11 | S→C | Confirms creation; returns queue name, message count, consumer count | ✅ Real counts (based on current storage offsets and shared consume-group member count) |
| queue.bind | 50.20 | C→S | Bind a queue to an exchange (with a routing-key) | ✅ |
| queue.bind-ok | 50.21 | S→C | Confirms the binding | ✅ |
| queue.unbind | 50.50 | C→S | Remove the binding between a queue and an exchange | ✅ |
| queue.unbind-ok | 50.51 | S→C | Confirms the unbind | ✅ |
| queue.purge | 50.30 | C→S | Purge all messages from a queue | ✅ |
| queue.purge-ok | 50.31 | S→C | Confirms the purge; returns the number of purged messages | ✅ |
| queue.delete | 50.40 | C→S | Delete a queue (if-unused / if-empty options) | ✅ |
| queue.delete-ok | 50.41 | S→C | Confirms deletion; returns the number of deleted messages | ✅ |

---

## 5. Basic Class

The Basic class is the heart of AMQP 0-9-1, covering all message publish, delivery, and acknowledgement logic.

### 5.1 Consumer Management

| Class.Method | ID | Direction | Description | Status |
|--------------|------|------|------|------|
| basic.qos | 60.10 | C→S | Set prefetch (prefetch-size, prefetch-count, global) | 🟡 Enforced when the consumer is co-located with the queue leader; best-effort across nodes; `prefetch-size` has no effect |
| basic.qos-ok | 60.11 | S→C | Confirms the QoS setting | ✅ |
| basic.consume | 60.20 | C→S | Register a consumer, start push-mode delivery (no-local/no-ack/exclusive) | ✅ `exclusive` is enforced; `no-local` has no effect (matches RabbitMQ classic queues) |
| basic.consume-ok | 60.21 | S→C | Returns the consumer-tag | ✅ Broker generates one if the client leaves it blank |
| basic.cancel | 60.30 | C→S | Cancel a consumer | ✅ |
| basic.cancel-ok | 60.31 | S→C | Confirms the cancellation | ✅ |

### 5.2 Publishing

| Class.Method | ID | Direction | Description | Status |
|--------------|------|------|------|------|
| basic.publish | 60.40 | C→S | Publish a message (exchange, routing-key, mandatory, immediate), followed by Content Header + Body frames | ✅ `immediate` is a deprecated flag and gets no special handling (matches modern RabbitMQ) |
| basic.return | 60.50 | S→C | Return an unroutable message (triggered by the `mandatory` flag) | ✅ |

### 5.3 Delivery

| Class.Method | ID | Direction | Description | Status |
|--------------|------|------|------|------|
| basic.deliver | 60.60 | S→C | Broker pushes a message to a consumer (push mode), followed by Content Header + Body frames | ✅ |
| basic.get | 60.70 | C→S | Synchronously pull one message (pull mode) | ✅ Auto-forwarded across nodes to the queue leader |
| basic.get-ok | 60.71 | S→C | Returns the message, followed by Content Header + Body frames | ✅ |
| basic.get-empty | 60.72 | S→C | Response when the queue is empty | ✅ |

### 5.4 Acknowledgement

| Class.Method | ID | Direction | Description | Status |
|--------------|------|------|------|------|
| basic.ack | 60.80 | C→S | Acknowledge a message as processed (supports batch ack via `multiple`) | ✅ |
| basic.reject | 60.90 | C→S | Reject a message (`requeue=true` puts it back, `false` discards it) | ✅ |
| basic.recover-async | 60.100 | C→S | Ask the broker to redeliver all unacked messages (fire-and-forget) | ✅ |
| basic.recover | 60.110 | C→S | Ask the broker to redeliver all unacked messages | ✅ |
| basic.recover-ok | 60.111 | S→C | Confirms the recover | ✅ |
| basic.nack | 60.120 | C→S | Batch-reject messages (RabbitMQ extension) | ✅ |

---

## 6. Tx Class (No Real Semantics)

| Class.Method | ID | Direction | Description | Status |
|--------------|------|------|------|------|
| tx.select | 90.10 | C→S | Enter transaction mode | 🟡 Handshake only, replies select-ok |
| tx.select-ok | 90.11 | S→C | Confirms transaction mode is on | 🟡 |
| tx.commit | 90.20 | C→S | Commit a transaction (publish + ack take effect atomically) | 🟡 Replies commit-ok only, no real atomic commit |
| tx.commit-ok | 90.21 | S→C | Confirms the commit | 🟡 |
| tx.rollback | 90.30 | C→S | Roll back a transaction | 🟡 Replies rollback-ok only, no real rollback |
| tx.rollback-ok | 90.31 | S→C | Confirms the rollback | 🟡 |

> After `Tx.Select`, publishes and acks still take effect **immediately** — they are not buffered until commit, and there is no rollback undo. For reliable publishing, use [Publisher Confirms](./PublisherConfirms.md) instead.

---

## 7. Confirm Class (Publisher Confirms, RabbitMQ Extension)

| Class.Method | ID | Direction | Description | Status |
|--------------|------|------|------|------|
| confirm.select | 85.10 | C→S | Enable Publisher Confirm mode | ✅ |
| confirm.select-ok | 85.11 | S→C | Confirms it's enabled | ✅ |

Once enabled, every `basic.publish` on that channel receives a matching `basic.ack` (success) or `basic.nack` (failure) once the message is durably written — see [Publisher Confirms](./PublisherConfirms.md).

---

## Broker Core Business Logic

About half of AMQP 0-9-1's methods are `*-ok` acknowledgements the broker can construct directly. The real business logic lives in:

| Core capability | Methods involved | Status |
|----------|-------------|------|
| **Authentication** | connection.start / start-ok | ✅ SASL PLAIN |
| **Routing** | exchange.declare + queue.bind + basic.publish | ✅ direct/fanout/topic/headers + exchange-chain bindings |
| **Shared queues / push delivery** | basic.consume + basic.deliver | ✅ Driven by the shared consume group's leader — see [Shared Queue Group](./SharedQueueGroup.md) |
| **Acknowledgement** | basic.ack / reject / nack / recover | ✅ Drives message state transitions (unacked → acked / requeued) |
| **Reliable publishing** | confirm.select + basic.ack/nack | ✅ See [Publisher Confirms](./PublisherConfirms.md) |
| **Prefetch flow control** | basic.qos | 🟡 Enforced locally, best-effort across nodes |
| **Transactions** | tx.select / commit / rollback | 🟡 Handshake only, no real semantics |

## Further Reading

- [Core Concepts](./AMQPCoreConcepts.md)
- [System Architecture](./SystemArchitecture.md)
- [Compatibility & Limitations](./Compatibility-and-Limitations.md)
- [Roadmap](./Roadmap.md)
