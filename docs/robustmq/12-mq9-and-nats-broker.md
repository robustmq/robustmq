# 12. mq9-core & nats-broker — Deep Dive

This chapter explains the two crates that together form RobustMQ's **NATS-compatible
protocol surface** and its **agent-oriented messaging extension ("mq9")**:

- [`src/mq9-core`](../../src/mq9-core) — the *contract* crate: the mq9 subject grammar and
  request/reply JSON DTOs. Pure types, no I/O.
- [`src/nats-broker`](../../src/nats-broker) — the *runtime* crate: a full NATS Core server
  (TCP / TLS / WS / WSS), plus the mq9 dispatcher built on top of NATS subjects, plus
  scaffolding for JetStream, KV, and Object store.

`nats-broker` depends on `mq9-core`. Both depend on the shared kernel
(`broker-core`, `storage-adapter`, `network-server`, `protocol`, `grpc-clients`,
`delay-message`, `common-*`).

---

## 12.1 Why two crates?

mq9 is a *protocol-on-a-protocol*: every mq9 operation is encoded as a normal NATS PUB on
a reserved subject namespace (`$mq9.AI.*`). Keeping the subject parser and DTOs in their
own crate lets:

- the NATS broker import them without circular deps;
- the gRPC server (Meta Service) and CLI tools reuse the same `Mq9Command` / reply types
  when introspecting or testing;
- the contract be versioned independently of the runtime.

```
+----------------------+        depends on        +----------------------+
|  src/nats-broker     | -----------------------> |  src/mq9-core        |
|  (runtime / server)  |                          |  (subjects + DTOs)   |
+----------------------+                          +----------------------+
        |                                                   |
        | uses                                              | used by CLI,
        v                                                   v Meta Service
  broker-core, storage-adapter, network-server,       admin-server, tests
  delay-message, common-*, protocol/nats
```

---

## 12.2 `mq9-core` — the contract crate

Crate: [src/mq9-core](../../src/mq9-core)

```
src/mq9-core/src
├── lib.rs        # re-exports `command` and `protocol`
├── command.rs    # `Mq9Command` enum + subject parser / formatter
└── protocol.rs   # request / reply structs (serde JSON)
```

Cargo manifest highlights ([Cargo.toml](../../src/mq9-core/Cargo.toml)):
`serde`, `serde_json`, `bytes`, `tokio`, `metadata-struct`, `storage-adapter`,
`broker-core`, `grpc-clients`, `common-base`, `common-config`. No protocol/codec
dependency — it is *pure data*.

### 12.2.1 Subject grammar — `Mq9Command`

Defined in [command.rs](../../src/mq9-core/src/command.rs). All mq9 traffic flows under a
single prefix:

```
$mq9.AI.<NAMESPACE>.<VERB>[.<arg>...]
```

| Subject                                              | Variant                              |
|------------------------------------------------------|--------------------------------------|
| `$mq9.AI.MAILBOX.CREATE`                             | `MailboxCreate`                      |
| `$mq9.AI.MSG.SEND.{mail_address}`                    | `MsgSend { mail_address }`           |
| `$mq9.AI.MSG.FETCH.{mail_address}`                   | `MsgFetch { mail_address }`          |
| `$mq9.AI.MSG.ACK.{mail_address}`                     | `MsgAck { mail_address }`            |
| `$mq9.AI.MSG.QUERY.{mail_address}`                   | `MsgQuery { mail_address }`          |
| `$mq9.AI.MSG.DELETE.{mail_address}.{msg_id}`         | `MsgDelete { mail_address, msg_id }` |
| `$mq9.AI.AGENT.REGISTER` / `UNREGISTER` / `REPORT` / `DISCOVER` | `AgentRegister` / ... etc.           |

The crate exposes three helpers:

- `Mq9Command::is_mq9_subject(s)` — cheap prefix check used by the NATS publish path.
- `Mq9Command::parse(s)` — `Option<Mq9Command>`; consumed by the broker's mq9 dispatcher.
- `Mq9Command::to_subject()` / `Display` — symmetric formatter for tests and clients.

`MsgSend` notably carries metadata in **NATS headers** rather than the subject (the
priority lives in the `mq9-priority` header — see §12.3.5).

### 12.2.2 DTOs — `protocol.rs`

[protocol.rs](../../src/mq9-core/src/protocol.rs) defines pure `serde` request / reply
types. Highlights:

- `MailboxCreateReq` / `MailboxCreateReply` — `name`, `ttl`, `desc` in; `mail_address`
  out.
- `MsgFetchReq` with `DeliverPolicy` (`Earliest | Latest | FromTime | FromId`), optional
  `group_name`, `force_deliver`, and `MsgFetchConfig { num_msgs, max_wait_ms }`. When
  `group_name` is omitted the broker uses a transient random group (no offset commit).
- `MsgFetchReply` returns a `Vec<MsgItem>` (`msg_id`, `payload`, `priority`, optional
  `header`, `create_time`).
- `MsgAckReq` / `MsgAckReply`, `MsgQueryReq` / `MsgQueryReply`,
  `MsgDeleteReply { deleted: bool }`.
- Agent management: `AgentRegisterReq`, `AgentReportReq`, `AgentDiscoverReq`
  (text / semantic search with pagination) and matching `*Reply` types. `AgentDiscoverReply`
  is intentionally schemaless (`Vec<serde_json::Value>`).
- `err_reply(msg)` — canonical JSON `{"error": "..."}` returned when a mq9 handler fails.

Every reply struct carries an `error: String` field (empty on success). This shape is
what mq9 *clients* see on the NATS reply subject.

---

## 12.3 `nats-broker` — the runtime crate

Crate: [src/nats-broker](../../src/nats-broker)

```
src/nats-broker/src
├── lib.rs
├── broker.rs        # NatsBrokerServer (top-level lifecycle)
├── server/          # TCP/TLS/WS/WSS listeners (network-server façade)
├── handler/         # per-packet command dispatcher (impls `Command`)
├── nats/            # NATS Core protocol handlers (CONNECT/PUB/SUB/PING/PONG)
├── mq9/             # mq9 dispatcher + per-verb handlers (built on NATS PUB)
├── jstream/         # JetStream / KV / Object-store subject taxonomy + handlers
├── push/            # subscription manager + fanout / queue-group push threads
├── storage/         # gRPC-backed metadata storage (mails, agents, subscriptions)
└── core/            # caches, error types, security, tenant, subject helpers
```

Cargo manifest highlights ([Cargo.toml](../../src/nats-broker/Cargo.toml)): `network-server`,
`protocol`, `broker-core`, `storage-adapter`, `delay-message`, `rate-limit`,
`grpc-clients`, `common-{base,security,config}`, `mq9-core`, `a2a-types`, `dashmap`,
`tonic`, `tokio`.

### 12.3.1 Top-level wiring — `broker.rs`

[broker.rs](../../src/nats-broker/src/broker.rs) defines:

- `NatsBrokerServerParams` — the dependency bundle handed in by `broker-server` on boot:
  caches, subscribe manager, connection manager, client pool, broker (node) cache,
  global rate limiter, task supervisor, broadcast stop channel, request channel, storage
  driver manager, security manager, delay message manager.
- `NatsBrokerServer::new(...)` constructs the listener (`NatsServer`) and the
  client keep-alive watcher (`NatsClientKeepAlive`).
- `start()` does three things and then blocks on stop:
  1. `start_sub_task(...)` — boots the parse thread + fanout/queue push workers
     (see §12.3.6).
  2. Spawns `NATSClientKeepAlive` via the shared `TaskSupervisor`.
  3. Starts the TCP/TLS/WS/WSS listeners.

Ports come from `broker_config().nats_runtime.{tcp_port,tls_port,ws_port,wss_port}`.

### 12.3.2 Listeners — `server/`

[server/mod.rs](../../src/nats-broker/src/server/mod.rs) wraps the shared
`network-server` primitives:

- One `TcpServer` instance for plain TCP, one for TLS, both parameterized with
  `RobustMQProtocol::NATS` so the framing/codec is selected automatically.
- One `WebSocketServer` covering both WS and WSS (also `RobustMQProtocol::NATS`).

`start()` starts the TCP listeners synchronously and spawns the WS/WSS tasks.

### 12.3.3 Packet dispatcher — `handler/command.rs`

[handler/command.rs](../../src/nats-broker/src/handler/command.rs) implements the
`network_server::command::Command` trait via `NatsHandlerCommand`. For every inbound
packet it:

1. Bumps the per-connection heartbeat (`connection_manager.report_heartbeat`).
2. Builds a cheap `NatsProcessContext` (clones of `Arc`s).
3. Reads the connection's `verbose` flag from the cache (NATS-spec `+OK` echo).
4. Dispatches by variant:

| NATS packet          | Handler                                                    | Notes                          |
|----------------------|------------------------------------------------------------|--------------------------------|
| `Connect`            | [`nats::connect::process_connect`](../../src/nats-broker/src/nats/connect.rs)       | sets `verbose` from request    |
| `Pub` / `HPub`       | [`nats::publish::process_pub`](../../src/nats-broker/src/nats/publish.rs)           | routes to mq9 if `$mq9.AI.*`   |
| `Sub`                | [`nats::subscribe::process_sub`](../../src/nats-broker/src/nats/subscribe.rs)       |                                |
| `Unsub`              | [`nats::subscribe::process_unsub`](../../src/nats-broker/src/nats/subscribe.rs)     |                                |
| `Ping` / `Pong`      | [`nats::ping::process_*`](../../src/nats-broker/src/nats/ping.rs)                   |                                |
| `Info/Msg/HMsg/Ok/Err` | — (server-to-client only)                                |                                |

The dispatcher converts each handler's outcome to an optional `NatsPacket` response
(verbose `+OK` only when configured, errors as `-ERR`).

### 12.3.4 NATS Core protocol — `nats/`

[nats/mod.rs](../../src/nats-broker/src/nats/mod.rs) re-exports `connect`, `ping`,
`publish`, `subscribe`. The interesting ones:

#### `publish.rs` — universal publish entry point

[publish.rs](../../src/nats-broker/src/nats/publish.rs):

```text
process_pub:
    if auth_required && !is_login        -> -ERR Authorization Violation
    if is_inbox_subject(subject)         -> (todo: client reply path)
    if Mq9Command::is_mq9_subject(...)   -> delegate to mq9::process::mq9_command
    else                                 -> process_pub0  (write to storage)
```

`process_pub0` is the NATS-Core write path:

1. `try_get_or_init_subject` ensures a topic / shard exists for `subject` (lazy create).
2. Build an `AdapterWriteRecord { payload, tags = [subject_message_tag(...)],
   protocol_data = StorageRecordProtocolDataNats { reply_to, header } }`.
3. `MessageStorage::new(...).write(tenant, subject, vec![record])` persists through
   `StorageDriverManager`.

This is what makes NATS PUBs *durable* in RobustMQ — they hit the same `StorageAdapter`
plane every other protocol uses.

#### `subscribe.rs` — SUB / UNSUB

[subscribe.rs](../../src/nats-broker/src/nats/subscribe.rs) supports:

- Plain subjects → recorded as a `NatsSubscribe` in
  [`NatsSubscribeStorage`](../../src/nats-broker/src/storage/subscribe.rs) (gRPC into
  Meta Service) and pushed into `NatsSubscribeManager` so the parse + push threads pick it
  up.
- Inbox subjects (`is_inbox_subject`) → registered into `NatsCacheManager.inbox_data` for
  fast reply routing (no shard/storage write).
- Queue groups → registered as `ShareGroupMember { params: ShareGroupParams::NATS(...) }`
  to reuse the shared-subscription kernel.
- The constant tag function `subject_message_tag(tenant, subject) = "{tenant}_{subject}"`
  is the key that ties stored records back to subscribers.

### 12.3.5 mq9 dispatcher — `mq9/`

[mq9/mod.rs](../../src/nats-broker/src/mq9/mod.rs) re-exports per-verb handlers and
two small helpers (`scoped_key`, `scoped_tag`) that namespace storage keys/tags by
`{tenant}/{mail_address}/{key}`.

The entry is [mq9/process.rs::mq9_command](../../src/nats-broker/src/mq9/process.rs):

```text
mq9_command(ctx, subject, reply_to, headers, payload) -> Option<NatsPacket>
    parsed = Mq9Command::parse(subject)?     // unrecognized -> -ERR
    response_json = match parsed {
        MailboxCreate     => process_create(...)
        MsgSend { addr }  => process_send(ctx, addr, headers, reply_to, payload)
        MsgFetch  { addr } => process_fetch(...)
        MsgAck    { addr } => process_ack(...)
        MsgQuery  { addr } => process_query(...)
        MsgDelete{addr,id} => process_delete(...)
        AgentRegister     => process_agent_register(...)
        AgentUnregister   => process_agent_unregister(...)
        AgentReport       => process_agent_report(...)
        AgentDiscover     => process_agent_discover(...)
    } |> serde_json::to_string  or  err_reply(e)
    if let Some(reply_subject) = reply_to {
        reply_nats_packet(ctx, reply_subject, response_json)
    }
    None                                     // never echoes Ok inline
```

`reply_nats_packet` looks up the SID for the inbox in `NatsCacheManager.inbox_data` and
writes a `NatsPacket::Msg { subject, sid, payload, reply_to: None }` straight to the
caller's connection through [`write_nats_packet`](../../src/nats-broker/src/core/write_client.rs).

#### `mq9/send.rs` — header-driven message ingestion

[send.rs](../../src/nats-broker/src/mq9/send.rs) parses the NATS HMSG header block for
five mq9-reserved headers:

| Header          | Meaning                                                          |
|-----------------|------------------------------------------------------------------|
| `mq9-key`       | Compaction / dedup key — recorded via `AdapterWriteRecord::with_key` |
| `mq9-delay`     | Seconds to delay; delegates to `delay_message_manager`           |
| `mq9-ttl`       | Seconds; sets `expire_at = now + ttl`                            |
| `mq9-tags`      | Comma-separated user tags, scoped per tenant/mailbox             |
| `mq9-priority`  | `normal | urgent | critical` — controls the storage priority lane |

The handler validates the mailbox exists in `NatsCacheManager.mail_info`, ensures the
subject/shard is initialised, then writes a single `AdapterWriteRecord` whose
`protocol_data.mq9 = StorageRecordProtocolDataMq9 { priority, header, reply_to }`.
If `mq9-delay` is set, no immediate write happens; the record is parked in
`delay-message`.

#### `mq9/fetch.rs` — priority-aware pull with consumer groups

[fetch.rs](../../src/nats-broker/src/mq9/fetch.rs) uses `PriorityGroupConsumer` from
`storage-adapter`:

- `group_name` set → durable group; offset commit is gated on a later `MsgAck`.
- `group_name` absent → transient UUID group (best-effort, no commit).
- `force_deliver = true` → `force_reset_offset(...)` computes the new offset per shard
  from `DeliverPolicy` (Earliest / Latest / FromTime / FromId) and writes it via
  `set_current_offsets`, so the next pull resumes from there irrespective of any
  previously committed offset.
- Long-poll: when there are no records and `max_wait_ms > 0`, the handler `sleep`s before
  returning an empty reply.

The companion `process_ack` (same file) advances the group offset by `msg_id`.

#### Other mq9 verbs

- `mq9/create.rs` — creates an `MQ9Mail`, persists it through `Mq9MailStorage` (gRPC →
  Meta) and inserts it into `NatsCacheManager.mail_info`.
- `mq9/delete.rs` / `mq9/query.rs` — direct reads/deletes against `StorageAdapter` using
  the same scoped-tag/key convention.
- `mq9/agent.rs` — register/unregister/report/discover, persisted via
  [`Mq9AgentStorage`](../../src/nats-broker/src/storage/agent.rs). Discover supports
  text and semantic search through `placement_search_mq9_agent`.

### 12.3.6 Push pipeline — `push/`

[push/mod.rs](../../src/nats-broker/src/push/mod.rs) is the subscription dispatch
engine.

`NatsSubscribeManager` ([manager.rs](../../src/nats-broker/src/push/manager.rs)) holds:

- `subscribe_list: DashMap<"{connect_id}#{sid}", NatsSubscribe>` — all live SUBs.
- `nats_core_fanout_push: NatsBucketsManager` — buckets for wildcard / fanout subjects.
- `nats_core_queue_push: DashMap<"{tenant}#{queue_group}#{subject}", NatsBucketsManager>`
  — per-queue-group buckets.
- `nats_core_queue_push_thread` — runtime info for live queue push tasks
  (`QueuePushThreadInfo { stop_tx, total_pushed, last_pull_time }`).
- `not_push_client` — connections temporarily excluded from push (slow consumer).
- A `parse_sender` channel feeding [`parse.rs`](../../src/nats-broker/src/push/parse.rs).

`start_sub_task` (called from `NatsBrokerServer::start`) wires two stages:

1. **Parse stage** — `start_subscribe_parse_thread` listens on the parse channel and
   classifies each new SUB into the right bucket (fanout vs queue group).
2. **Push stage** — `start_sub_push_thread` ([thread.rs](../../src/nats-broker/src/push/thread.rs)):
    - Spawns `push_thread_num` fanout workers, each owning a `FanoutPushManager`
      ([nats_fanout.rs](../../src/nats-broker/src/push/nats_fanout.rs)).
    - Starts a queue watcher that lazily spawns one `QueuePushManager`
      ([nats_queue.rs](../../src/nats-broker/src/push/nats_queue.rs)) per queue group as
      members appear.

Each push manager reads records via `StorageDriverManager`, matches them against the
bucket's subscribers (`NatsBucketsManager` in [buckets.rs](../../src/nats-broker/src/push/buckets.rs))
and writes `NatsPacket::Msg` / `HMsg` straight to client connections.

### 12.3.7 Caches & helpers — `core/`

| File                                                                                 | Role                                                              |
|--------------------------------------------------------------------------------------|-------------------------------------------------------------------|
| [cache.rs](../../src/nats-broker/src/core/cache.rs)                                   | `NatsCacheManager`: connections, mails, inboxes, agents (`DashMap`) |
| [connection.rs](../../src/nats-broker/src/core/connection.rs)                         | `NatsConnection` per-client state (`verbose`, login user, etc.)   |
| [dynamic_cache.rs](../../src/nats-broker/src/core/dynamic_cache.rs)                   | hot-reload of dynamic NATS config                                 |
| [error.rs](../../src/nats-broker/src/core/error.rs)                                   | `NatsBrokerError`, `NatsProtocolError` (with NATS-spec messages)  |
| [keep_alive.rs](../../src/nats-broker/src/core/keep_alive.rs)                         | `NatsClientKeepAlive::start_heartbeat_check`                      |
| [mail.rs](../../src/nats-broker/src/core/mail.rs)                                     | mail-address helpers (allocation, validation)                     |
| [delay.rs](../../src/nats-broker/src/core/delay.rs)                                   | `save_delay_message` bridge into the `delay-message` crate        |
| [queue_name.rs](../../src/nats-broker/src/core/queue_name.rs)                         | queue-group member add/remove helpers                             |
| [security.rs](../../src/nats-broker/src/core/security.rs)                             | thin wrappers over `common-security::SecurityManager`             |
| [subject.rs](../../src/nats-broker/src/core/subject.rs)                               | `is_inbox_subject`, `try_get_or_init_subject` (lazy shard create) |
| [tenant.rs](../../src/nats-broker/src/core/tenant.rs)                                 | `get_tenant()` (currently `DEFAULT_TENANT`)                       |
| [topic.rs](../../src/nats-broker/src/core/topic.rs)                                   | NATS-subject ↔ topic naming                                       |
| [write_client.rs](../../src/nats-broker/src/core/write_client.rs)                     | `write_nats_packet(connection_manager, connect_id, pkt)`          |

### 12.3.8 Metadata storage — `storage/`

[storage/mod.rs](../../src/nats-broker/src/storage/mod.rs) wraps the Meta Service gRPC
calls behind small typed stores:

- [`storage/mail.rs`](../../src/nats-broker/src/storage/mail.rs) — `Mq9MailStorage`
  (`create / delete / list`).
- [`storage/agent.rs`](../../src/nats-broker/src/storage/agent.rs) — `Mq9AgentStorage`
  (`create / delete / list / search_by_text / search_by_semantic`).
- [`storage/subscribe.rs`](../../src/nats-broker/src/storage/subscribe.rs) —
  `NatsSubscribeStorage` (`save`, `list`, …).
- [`storage/message.rs`](../../src/nats-broker/src/storage/message.rs) —
  `MessageStorage` thin wrapper around `StorageDriverManager` for protocol-agnostic
  reads/writes.

None of these touch RocksDB directly — they go through `ClientPool` against Meta Service
or through `StorageAdapter`.

### 12.3.9 JetStream / KV / Object — `jstream/`

[jstream/mod.rs](../../src/nats-broker/src/jstream/mod.rs) declares the module skeleton.
The taxonomy lives in [jstream/command.rs](../../src/nats-broker/src/jstream/command.rs)
as `JsCommand`, covering all the standard `$JS.API.*`, `$JS.ACK.*`, `$JS.EVENT.*`,
`$KV.*`, `$OBJ.*` subjects (streams, consumers, direct get, KV bucket ops, object-store
ops, advisory events). The handler files
([stream.rs](../../src/nats-broker/src/jstream/stream.rs),
[consumer.rs](../../src/nats-broker/src/jstream/consumer.rs),
[ack.rs](../../src/nats-broker/src/jstream/ack.rs),
[direct.rs](../../src/nats-broker/src/jstream/direct.rs),
[event.rs](../../src/nats-broker/src/jstream/event.rs),
[kv.rs](../../src/nats-broker/src/jstream/kv.rs),
[object.rs](../../src/nats-broker/src/jstream/object.rs),
[info.rs](../../src/nats-broker/src/jstream/info.rs),
[process.rs](../../src/nats-broker/src/jstream/process.rs),
[protocol.rs](../../src/nats-broker/src/jstream/protocol.rs)) are wired but
implementation-in-progress; they will share the same dispatch model as mq9 (subject parse
→ verb handler → JSON reply on the inbox).

---

## 12.4 End-to-end: an mq9 `MSG.SEND` request

A client publishes:

```
HPUB $mq9.AI.MSG.SEND.task.001  _INBOX.xyz  <hdrs>  <payload>
NATS/1.0
mq9-priority: urgent
mq9-key: order-42
mq9-tags: billing,vip
```

What happens:

1. `network-server` decodes a `RobustMQPacket::NATS(NatsPacket::HPub { ... })` and calls
   `NatsHandlerCommand::apply` ([handler/command.rs](../../src/nats-broker/src/handler/command.rs)).
2. `apply` refreshes heartbeat, builds `NatsProcessContext`, calls
   `nats::publish::process_pub`.
3. `process_pub` sees the subject starts with `$mq9.AI`, routes to
   `mq9::process::mq9_command`.
4. `Mq9Command::parse` returns `MsgSend { mail_address: "task.001" }`.
5. `mq9::send::process_send` validates the mailbox exists in `NatsCacheManager`, parses
   the headers (`Mq9Headers`), lazy-creates the subject/shard, builds an
   `AdapterWriteRecord` with `priority=urgent`, scoped tags, `mq9` protocol data,
   compaction key `default/task.001/order-42`, then writes via `MessageStorage`.
6. The returned `MsgSendReply { error: "", msg_id }` is `serde_json::to_string`ed and
   posted back on `_INBOX.xyz` via `reply_nats_packet` → `write_nats_packet`.
7. Concurrently, the parse + fanout/queue push threads notice the new record (through
   the `StorageDriverManager` change stream) and deliver it to any matching NATS Core
   subscribers / queue-group members on `task.001`.

The mq9 client receives a JSON reply containing the new `msg_id`; any plain NATS consumer
subscribed to the same subject sees the original payload, including its mq9 headers.

---

## 12.5 Where it plugs into the rest of RobustMQ

- **Boot**: `broker-server` constructs `NatsBrokerServerParams` and calls
  `NatsBrokerServer::start` once `nats_runtime.enable = true` and the node has the
  broker role. See [03-startup-flow.md](03-startup-flow.md) for the role-gating model.
- **Storage**: every NATS publish and every mq9 message uses the same
  `StorageAdapter` write path documented in
  [05-storage-layer.md](05-storage-layer.md). The `protocol_data` discriminant on each
  record marks origin (mqtt / nats / mq9) so cross-protocol consumption is possible.
- **Meta**: mailboxes, agents and subscriptions are durable in Meta Service
  ([04-meta-service.md](04-meta-service.md)) via the typed accessors in
  [`storage/`](../../src/nats-broker/src/storage/).
- **Push**: the per-bucket worker model is a NATS-flavoured sibling of the MQTT
  dispatch kernel described in [06-brokers.md](06-brokers.md).
- **Delayed delivery**: the `mq9-delay` header funnels writes into the
  [`delay-message`](../../src/delay-message) crate, the same scheduler MQTT delayed
  publish uses.

---

## 12.6 Quick reference — file → responsibility

### `src/mq9-core/src/`

| File          | Responsibility                                             |
|---------------|------------------------------------------------------------|
| `lib.rs`      | re-exports `command`, `protocol`                           |
| `command.rs`  | `Mq9Command` enum, subject parser/formatter, tests         |
| `protocol.rs` | request/reply structs, `DeliverPolicy`, `err_reply` helper |

### `src/nats-broker/src/`

| File / dir              | Responsibility                                                       |
|-------------------------|----------------------------------------------------------------------|
| `broker.rs`             | `NatsBrokerServer` lifecycle                                         |
| `server/`               | TCP/TLS/WS/WSS listener wiring                                       |
| `handler/command.rs`    | `Command` impl, per-packet dispatch                                  |
| `nats/connect.rs`       | CONNECT — auth, verbose, INFO reply                                  |
| `nats/publish.rs`       | PUB/HPUB — auth, mq9 routing, storage write                          |
| `nats/subscribe.rs`     | SUB/UNSUB — register, queue groups, inbox fast-path                  |
| `nats/ping.rs`          | PING/PONG                                                            |
| `mq9/process.rs`        | mq9 dispatcher (verb → handler → JSON reply)                         |
| `mq9/send.rs`           | header parsing, delay, priority, storage write                       |
| `mq9/fetch.rs`          | priority pull, durable/transient groups, force-reset, long-poll      |
| `mq9/create.rs`         | mailbox creation (`MQ9Mail` + cache + Meta)                          |
| `mq9/delete.rs`         | delete one message by id                                             |
| `mq9/query.rs`          | tag/time-range query                                                 |
| `mq9/agent.rs`          | agent register / unregister / report / discover                      |
| `jstream/command.rs`    | full JetStream/KV/Object subject taxonomy                            |
| `jstream/{stream,consumer,ack,direct,event,kv,object,info,process,protocol}.rs` | per-API handlers (work in progress) |
| `push/manager.rs`       | `NatsSubscribeManager` (subscribe index, buckets, parse channel)     |
| `push/parse.rs`         | classify SUBs into fanout/queue buckets                              |
| `push/thread.rs`        | spawn fanout workers + queue-group watcher                           |
| `push/nats_fanout.rs`   | wildcard / fanout push worker                                        |
| `push/nats_queue.rs`    | queue-group (load-balanced) push worker                              |
| `push/buckets.rs`       | `NatsBucketsManager` — subscriber index per bucket                   |
| `push/common.rs`        | shared push utilities                                                |
| `core/cache.rs`         | `NatsCacheManager` (connections / mails / inboxes / agents)          |
| `core/keep_alive.rs`    | client heartbeat checker                                             |
| `core/subject.rs`       | `is_inbox_subject`, `try_get_or_init_subject`                        |
| `core/delay.rs`         | bridge to `delay-message`                                            |
| `core/write_client.rs`  | server-initiated packet writer                                       |
| `core/error.rs`         | `NatsBrokerError`, `NatsProtocolError`                               |
| `storage/mail.rs`       | `Mq9MailStorage` (Meta gRPC)                                         |
| `storage/agent.rs`      | `Mq9AgentStorage` (Meta gRPC + search)                               |
| `storage/subscribe.rs`  | `NatsSubscribeStorage` (Meta gRPC)                                   |
| `storage/message.rs`    | `MessageStorage` (StorageAdapter façade)                             |
