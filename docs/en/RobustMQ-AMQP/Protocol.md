# RobustMQ AMQP Protocol Support

This document lists the AMQP 0.9.1 protocol methods that RobustMQ needs to support as an AMQP Broker, along with their priority and description.

References:
- [AMQP 0.9.1 Protocol Specification](https://www.rabbitmq.com/resources/specs/amqp0-9-1.pdf)
- [AMQP 0.9.1 XML Definition](https://www.rabbitmq.com/resources/specs/amqp0-9-1.xml)

---

## Protocol Basics

AMQP 0.9.1 runs over TCP and uses a frame-based transport. Each frame consists of a type, channel number, payload length, payload, and a frame-end byte (0xCE).

- Connection flow
![img](../../images/amqp-01.jpg)

- Produce & Consume flow

![img](../../images/amqp-02.jpg)

- Broker internal logic
![img](../../images/amqp-03.jpg)

### Frame Types

| Frame Type | Code | Description |
|------------|------|-------------|
| Method Frame | 1 | Control commands (all Class/Method pairs) |
| Content Header Frame | 2 | Message properties (content-type, delivery-mode, headers, etc.) |
| Content Body Frame | 3 | Message payload (may be split across multiple frames) |
| Heartbeat Frame | 8 | Keep-alive heartbeat |

Message publishing and delivery each require a **Method + Content Header + Content Body** frame sequence.

---

## 1. Connection Class (Required)

Connection-level handshake, all on channel=0. The broker initiates `start`, `secure`, `tune`, and `close`, and processes the client's responses.

| Class.Method | Code | Direction | Description | Supported |
|--------------|------|-----------|-------------|-----------|
| connection.start | 10.10 | S→C | Broker initiates handshake, advertises supported SASL mechanisms and locales | ❌ |
| connection.start-ok | 10.11 | C→S | Client selects SASL mechanism and sends authentication response | ❌ |
| connection.secure | 10.20 | S→C | Broker sends SASL challenge (for multi-step mechanisms) | ❌ |
| connection.secure-ok | 10.21 | C→S | Client responds to SASL challenge | ❌ |
| connection.tune | 10.30 | S→C | Broker proposes channel-max, frame-max, heartbeat parameters | ❌ |
| connection.tune-ok | 10.31 | C→S | Client confirms connection parameters | ❌ |
| connection.open | 10.40 | C→S | Client opens a virtual host | ❌ |
| connection.open-ok | 10.41 | S→C | Broker confirms vhost connection | ❌ |
| connection.close | 10.50 | Both | Either side initiates connection close (with error code) | ❌ |
| connection.close-ok | 10.51 | Both | Confirms close | ❌ |

---

## 2. Channel Class (Required)

Multiple channels can be multiplexed over a single TCP connection, each independently handling message flow.

| Class.Method | Code | Direction | Description | Supported |
|--------------|------|-----------|-------------|-----------|
| channel.open | 20.10 | C→S | Client opens a channel | ❌ |
| channel.open-ok | 20.11 | S→C | Broker confirms channel is open | ❌ |
| channel.flow | 20.20 | Both | Pause or resume message flow (back-pressure control) | ❌ |
| channel.flow-ok | 20.21 | Both | Confirms flow command | ❌ |
| channel.close | 20.40 | Both | Close channel (with error code) | ❌ |
| channel.close-ok | 20.41 | Both | Confirms close | ❌ |

---

## 3. Exchange Class (Required)

Exchanges are the core of message routing, supporting direct, fanout, topic, and headers types.

| Class.Method | Code | Direction | Description | Supported |
|--------------|------|-----------|-------------|-----------|
| exchange.declare | 40.10 | C→S | Create or verify an exchange (type/passive/durable/no-wait) | ❌ |
| exchange.declare-ok | 40.11 | S→C | Confirms creation | ❌ |
| exchange.delete | 40.20 | C→S | Delete an exchange (if-unused option) | ❌ |
| exchange.delete-ok | 40.21 | S→C | Confirms deletion | ❌ |

---

## 4. Queue Class (Required)

| Class.Method | Code | Direction | Description | Supported |
|--------------|------|-----------|-------------|-----------|
| queue.declare | 50.10 | C→S | Create or verify a queue (passive/durable/exclusive/auto-delete) | ❌ |
| queue.declare-ok | 50.11 | S→C | Confirms creation, returns queue name, message count, consumer count | ❌ |
| queue.bind | 50.20 | C→S | Bind a queue to an exchange (with routing-key) | ❌ |
| queue.bind-ok | 50.21 | S→C | Confirms binding | ❌ |
| queue.unbind | 50.50 | C→S | Remove a queue binding from an exchange | ❌ |
| queue.unbind-ok | 50.51 | S→C | Confirms unbinding | ❌ |
| queue.purge | 50.30 | C→S | Remove all unacknowledged messages from a queue | ❌ |
| queue.purge-ok | 50.31 | S→C | Confirms purge, returns message count removed | ❌ |
| queue.delete | 50.40 | C→S | Delete a queue (if-unused / if-empty options) | ❌ |
| queue.delete-ok | 50.41 | S→C | Confirms deletion, returns message count removed | ❌ |

---

## 5. Basic Class (Required)

The Basic class is the core of AMQP 0.9.1, covering message publishing, delivery, and acknowledgment.

### 5.1 Consumer Management

| Class.Method | Code | Direction | Description | Supported |
|--------------|------|-----------|-------------|-----------|
| basic.qos | 60.10 | C→S | Set prefetch (prefetch-size, prefetch-count, global) | ❌ |
| basic.qos-ok | 60.11 | S→C | Confirms QoS settings | ❌ |
| basic.consume | 60.20 | C→S | Register a consumer, start push-mode delivery (no-local/no-ack/exclusive) | ❌ |
| basic.consume-ok | 60.21 | S→C | Returns consumer-tag | ❌ |
| basic.cancel | 60.30 | C→S | Cancel a consumer | ❌ |
| basic.cancel-ok | 60.31 | S→C | Confirms cancellation | ❌ |

### 5.2 Message Publishing

| Class.Method | Code | Direction | Description | Supported |
|--------------|------|-----------|-------------|-----------|
| basic.publish | 60.40 | C→S | Publish a message (exchange, routing-key, mandatory, immediate), followed by Content Header + Body frames | ❌ |
| basic.return | 60.50 | S→C | Return an unroutable message to publisher (triggered by mandatory/immediate flags) | ❌ |

### 5.3 Message Delivery

| Class.Method | Code | Direction | Description | Supported |
|--------------|------|-----------|-------------|-----------|
| basic.deliver | 60.60 | S→C | Broker pushes message to consumer (push mode), followed by Content Header + Body frames | ❌ |
| basic.get | 60.70 | C→S | Synchronously pull one message (pull mode) | ❌ |
| basic.get-ok | 60.71 | S→C | Returns message, followed by Content Header + Body frames | ❌ |
| basic.get-empty | 60.72 | S→C | Response when queue is empty | ❌ |

### 5.4 Message Acknowledgment

| Class.Method | Code | Direction | Description | Supported |
|--------------|------|-----------|-------------|-----------|
| basic.ack | 60.80 | C→S | Acknowledge message(s) as processed (multiple flag for batch ack) | ❌ |
| basic.reject | 60.90 | C→S | Reject a message (requeue=true to requeue, false to discard) | ❌ |
| basic.recover | 60.110 | C→S | Ask broker to redeliver all unacknowledged messages | ❌ |
| basic.recover-ok | 60.111 | S→C | Confirms recover | ❌ |

---

## 6. Tx Class (Optional, Local Transactions)

| Class.Method | Code | Direction | Description | Supported |
|--------------|------|-----------|-------------|-----------|
| tx.select | 90.10 | C→S | Enable transaction mode | ❌ |
| tx.select-ok | 90.11 | S→C | Confirms transaction mode enabled | ❌ |
| tx.commit | 90.20 | C→S | Commit transaction (publish + ack atomically) | ❌ |
| tx.commit-ok | 90.21 | S→C | Confirms commit | ❌ |
| tx.rollback | 90.30 | C→S | Roll back transaction | ❌ |
| tx.rollback-ok | 90.31 | S→C | Confirms rollback | ❌ |

---

## 7. RabbitMQ Extensions (Optional)

The following are RabbitMQ private extensions to AMQP 0.9.1, not part of the standard spec, but widely used by mainstream clients:

| Extension | Description | Supported |
|-----------|-------------|-----------|
| **basic.nack** | Batch reject messages (standard reject only handles one at a time) | ❌ |
| **confirm.select / confirm.select-ok** | Publisher Confirm mode: broker sends ack/nack for each published message | ❌ |
| **exchange.bind / exchange.bind-ok** | Exchange-to-Exchange binding | ❌ |
| **exchange.unbind / exchange.unbind-ok** | Remove Exchange-to-Exchange binding | ❌ |

> Publisher Confirm is an almost universally required reliability feature in production — recommended to implement alongside basic.publish.

---

## Core Broker Business Logic

About half of the 53 methods in AMQP 0.9.1 are `*-ok` acknowledgment replies that the broker constructs and returns directly. The methods requiring real business logic are:

| Capability | Methods Involved | Description |
|------------|-----------------|-------------|
| **Authentication** | connection.start / start-ok / secure / secure-ok | SASL handshake, supporting PLAIN and AMQPLAIN mechanisms |
| **Routing** | exchange.declare + queue.bind + basic.publish | publish → exchange → binding → queue matching; supports direct/fanout/topic/headers |
| **Push Delivery** | basic.consume + basic.deliver | Maintain consumer registry, respect prefetch window, push messages to consumers |
| **Acknowledgment** | basic.ack / reject / nack / recover | Drive message state transitions (unacked → acked / requeued / dead-lettered) |
| **Transactions** | tx.select / commit / rollback | Atomicity guarantee for publish and ack operations (optional) |

---

## Implementation Roadmap

### Phase 1: Standard Client Compatibility

After implementing the following methods, standard AMQP clients (pika, amqplib, etc.) can send and receive messages:

```text
connection: start → start-ok → tune → tune-ok → open → open-ok
channel: open → open-ok
exchange: declare → declare-ok
queue: declare → declare-ok → bind → bind-ok
basic: publish(+Header+Body) → deliver(+Header+Body)
basic: consume → consume-ok → ack
connection/channel: close → close-ok
```

### Phase 2: Full Message Semantics

Add on top of Phase 1:

- basic.qos (prefetch flow control)
- basic.reject / basic.recover (message redelivery)
- basic.get / get-ok / get-empty (pull mode)
- basic.return (mandatory message return)
- queue.purge / delete, exchange.delete

### Phase 3: Reliability & Transactions

- Publisher Confirm (confirm.select + basic.ack/nack)
- Tx transactions (tx.select / commit / rollback)
- basic.nack (batch reject)
- Exchange-to-Exchange binding
