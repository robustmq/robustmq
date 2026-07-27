# Shared Queue Group

This is the most central design decision in RobustMQ's AMQP implementation: **every AMQP queue is internally a shared consume group**, and that shared consume-group infrastructure is the same one used by MQTT shared subscriptions and NATS queue groups — AMQP does not implement a separate queue engine.

## Why Reuse the Shared Consume Group

AMQP's "one queue, many competing consumers" and MQTT/NATS's "a group of clients sharing a subscription to the same topic" are fundamentally the same problem: one stream of data where only one of several consumers should get each message. RobustMQ already implements the "elect one leader node to pull and dispatch" mechanism for MQTT/NATS, and AMQP queues reuse it directly instead of reinventing it.

## How a Queue Maps to a Shared Consume Group

- The queue name is the shared consume group's name (`group_name = queue_name`).
- The group is created **on demand**: `Queue.Declare` only creates metadata and a storage shard; the actual shared consume group is created the first time a client calls `Basic.Get` or `Basic.Consume` on that queue.
- The group's leader is elected by meta-service based on cluster load; the election result (and the result of the group-creation operation itself) is persisted in Raft and visible to every node.

## Leader Resolution and Consistency at Creation Time

The first time a queue is accessed, RobustMQ needs to obtain (or create) its shared consume group to determine the leader. There's an easy pitfall here: **if you create the group and then separately issue a "read" to confirm what was just created, that read can land on a node that hasn't yet caught up on the latest data** (Raft's "commit" and "apply to the local state machine" are two separate phases, and different nodes can be briefly out of sync on apply progress).

RobustMQ's approach is to have the "create" operation itself return the authoritative result directly — the meta-service request that creates the shared consume group already computes the full result (including the elected leader) while handling the request, and hands that data straight back to the caller instead of requiring a separate follow-up read to confirm it. This avoids the "write, then immediately read back stale state" race.

## Competing Consumers

Multiple consumers (possibly connected to different nodes) calling `Basic.Consume` on the same queue are simply multiple members of that shared consume group:

- The push task on the queue's leader node round-robins across group members, handing the next message due for delivery to the next member that currently has spare prefetch quota.
- If the selected member can't currently accept a message (prefetch is full, or the connection is unavailable), the push task tries the next member in the group instead of blocking.
- If a consumer is on the same node as the push task (the leader), delivery happens locally; otherwise the message is sent via one gRPC call (`SendShareGroupMessage`) to whichever node and connection the consumer actually lives on.

`Basic.Get` uses the same leader mechanism: the request first resolves the queue's current leader, and if it isn't the local node, forwards the request (`FetchAmqpQueueMessage`) — fully transparent to the client.

## Known Limitation: Cross-Node QoS

`Basic.Qos`'s prefetch limit is enforced by having the queue's leader node check a consumer's current unacked-message count before deciding whether to keep delivering. This check is strongly consistent only when **the consumer is on the same node as the queue leader**, since the unacked count only lives in that node's local state. If a consumer is connected to a different node, the leader can't see its live unacked count, so prefetch enforcement is currently best-effort and not synchronized across nodes (this would require an extra RPC query or state replication, which isn't implemented yet).

## Further Reading

- [Core Concepts](./AMQPCoreConcepts.md)
- [Consuming](./Consuming.md)
- [System Architecture](./SystemArchitecture.md)
- [Compatibility & Limitations](./Compatibility-and-Limitations.md)
