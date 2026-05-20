# 11. End-to-End Flows

Three scenarios traced through the actual code paths.

## 11.1 MQTT PUBLISH → persisted record → Kafka FETCH

Scenario: an IoT device publishes `topic=sensors/room/1`, a data team consumes it from
Kafka as `topic=sensors.room.1`. Both protocols are hosted by the same RobustMQ cluster.

```
Device ── TCP ──▶ mqtt-broker/server  (listener accepts, decoder = protocol/mqtt)
                       │
                       ▼
            mqtt-broker/core/command.rs   (PUBLISH dispatch)
                       │
                       ├─▶ core/security.rs           (ACL via cache → broker-core)
                       ├─▶ core/topic.rs              (resolve/create topic; talks to Meta on miss)
                       ├─▶ core/qos.rs / pkid_manager (QoS 1/2 bookkeeping)
                       │
                       ▼
            storage-adapter::StorageAdapter::write(shard, [record])
                       │  driver = "engine"
                       ▼
            storage-engine/handler  ──▶  isr/  (replicate)  ──▶  filesegment/  (mmap append)
                       │
                       ▼
            offset returned, PUBACK sent back to device
                       │
                       ▼
            mqtt-broker/subscribe/manager.rs  (find matching subscribers)
                       │
                       ▼
            subscribe/push.rs ─▶ directly_push / share_push  (fan-out to MQTT subscribers)

  --- meanwhile ---

Kafka client ── TCP ──▶ kafka-broker/server  (Kafka frame decoder = protocol/kafka)
                              │
                              ▼
                    kafka-broker/handler/fetch.rs (FETCH for "sensors.room.1")
                              │
                              ▼
                    storage-adapter::read_by_offset(shard, offset, cfg)
                              │
                              ▼
                    SAME shard, SAME StorageRecord, returned in Kafka wire format
```

Key invariants exercised:

- Both protocols agreed on **shard naming** (resolved by `broker-core::topic`).
- Both protocols emit/consume **the same `StorageRecord`** (no translation).
- Replication, ACK, and offset semantics belong to the engine, not the brokers.

## 11.2 MQTT 5 delayed PUBLISH

```
PUBLISH (delay = 30s)
      │
      ▼
mqtt-broker/core/delay_message.rs
      │
      ▼
delay-message/manager.rs  (enqueue with deliver_at = now+30s)
      │
      ▼
StorageAdapter::write(internal_delay_shard, scheduled_record)
      │
      ▼
delay-message/pop.rs  (timer wheel from common/delay-task)
      │ deliver_at reached
      ▼
mqtt-broker re-injects message into the normal publish pipeline
```

`recover.rs` rebuilds the wheel from the internal shard on startup, so timers survive
crashes.

## 11.3 mq9 Agent registration, discovery, mailbox

```
Client (NATS) ──▶ nats-broker/server  (NATS line protocol)
                       │
                       ▼
              nats-broker/mq9/  (route on subject prefix $mq9.AI.*)

  $mq9.AI.AGENT.REGISTER
        │
        ▼
  mq9-core/protocol.rs : parse AgentCard
        │
        ├─▶ Meta : persist AgentCard
        └─▶ common/llm-engine : embed payload  ─▶ common/search-engine : index in LanceDB

  $mq9.AI.AGENT.DISCOVER  {semantic: "translate Chinese to English"}
        │
        ▼
  common/llm-engine : embed query
        │
        ▼
  common/search-engine : kNN  ─▶  return top-N AgentCards

  $mq9.AI.MAILBOX.CREATE
        │
        ▼
  Allocate dedicated shard via storage-adapter (priority-aware via consumer_priority.rs)

  $mq9.AI.MSG.SEND.<agent>
        │
        ▼
  StorageAdapter::write(mailbox_shard, record{priority, headers})

  $mq9.AI.MSG.FETCH.<agent>
        │
        ▼
  storage-adapter::consumer_priority : FETCH in critical → urgent → normal order

  $mq9.AI.MSG.ACK.<agent>
        │
        ▼
  common/group::OffsetManager : advance group offset (persisted via Meta)
```

Notes:

- mq9 reuses NATS framing and JetStream-style semantics — no separate protocol parser.
- Persistence is the same `StorageAdapter` used by MQTT and Kafka. Mailboxes are just
  shards with a 3-tier priority consumer.
- Discovery is the only path that touches the LLM/vector stack.

## 11.4 Cluster write path (Meta)

```
robust-ctl topic create ── gRPC ──▶ meta-service/server  (leader or follower)
                                          │ if follower: forward to leader
                                          ▼
                                meta-service/raft : Raft.propose(entry)
                                          │  quorum ack
                                          ▼
                                raft/route : apply entry → mutate state machine
                                          │
                                          ▼
                                emit change event
                                          │
                                          ▼
                broker-server/update_cache.rs on every broker refreshes local cache
                                          │
                                          ▼
                next publish to the new topic finds it in cache — no Meta RPC
```

This is why the hot publish path stays microseconds even though topic creation is a
strongly consistent operation.
