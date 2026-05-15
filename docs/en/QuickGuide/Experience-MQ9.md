# Experience mq9

## Prerequisites: Start the Broker

Follow [Quick Install](Quick-Install.md) to install RobustMQ, then start the service:

```bash
robust-server start
```

mq9 starts with RobustMQ — no additional configuration needed. It listens on the default NATS port `4222`.

---

## Install the NATS CLI

mq9 is built on NATS. All operations below use the NATS CLI:

```bash
# macOS
brew install nats-io/nats-tools/nats

# Linux / Windows
# See: https://docs.nats.io/using-nats/nats-tools/nats_cli
```

Set the server address once:

```bash
export NATS_URL=nats://localhost:4222
```

---

## Register an Agent

Register an Agent with its capability description. Other Agents can discover it by keyword or semantic intent:

```bash
nats request '$mq9.AI.AGENT.REGISTER' '{
  "name": "agent.translator",
  "mailbox": "agent.translator",
  "payload": "Multilingual translation; EN/ZH/JA/KO"
}'
```

---

## Discover Agents

Find Agents by semantic intent or keyword:

```bash
# Semantic search
nats request '$mq9.AI.AGENT.DISCOVER' '{"semantic":"translate Chinese to English","limit":5}'

# Full-text search
nats request '$mq9.AI.AGENT.DISCOVER' '{"text":"translator","limit":10}'
```

---

## Create a Mailbox

Each Agent gets a persistent mailbox. Messages wait until the Agent comes online:

```bash
nats request '$mq9.AI.MAILBOX.CREATE' '{"name":"agent.translator","ttl":3600}'
```

Response:

```json
{"error":"","mail_address":"agent.translator"}
```

---

## Send Messages

Send messages with three priority levels via the `mq9-priority` header:

```bash
# Highest priority — abort signals, urgent commands
nats request '$mq9.AI.MSG.SEND.agent.translator' \
  --header 'mq9-priority:critical' \
  '{"type":"abort","task_id":"t-001"}'

# Urgent
nats request '$mq9.AI.MSG.SEND.agent.translator' \
  --header 'mq9-priority:urgent' \
  '{"type":"interrupt","task_id":"t-002"}'

# Normal (default, no header needed)
nats request '$mq9.AI.MSG.SEND.agent.translator' \
  '{"type":"task","payload":"process dataset A"}'
```

Messages persist even if the recipient is offline.

---

## Fetch Messages (FETCH + ACK)

mq9 uses pull consumption. The client calls FETCH to retrieve messages — messages are returned in priority order (critical → urgent → normal):

```bash
nats request '$mq9.AI.MSG.FETCH.agent.translator' '{
  "group_name": "my-worker",
  "deliver": "earliest",
  "config": {"num_msgs": 10}
}'
```

After processing, ACK to advance the consumer group offset:

```bash
nats request '$mq9.AI.MSG.ACK.agent.translator' '{
  "group_name": "my-worker",
  "mail_address": "agent.translator",
  "msg_id": 1
}'
```

The next FETCH resumes from where the last ACK left off — no duplicate consumption.

---

## Next Steps

- **Full documentation** — [mq9.robustmq.com](https://mq9.robustmq.com)
- **Protocol reference** — [mq9.robustmq.com/docs/protocol](https://mq9.robustmq.com/docs/protocol)
- **SDK integration** — [mq9.robustmq.com/docs/sdk](https://mq9.robustmq.com/docs/sdk)
- **LangChain integration** — [mq9.robustmq.com/docs/langchain](https://mq9.robustmq.com/docs/langchain)
