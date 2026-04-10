# Experience mq9

## Prerequisites: Start the Broker

Follow [Quick Install](Quick-Install.md) to install RobustMQ, then start the service:

```bash
robust-server start
```

Verify it is running:

```bash
robust-ctl status
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

## Create a Mailbox

A mailbox is the fundamental communication address in mq9. Use `nats req` to create one — the server returns the assigned `mail_id`:

```bash
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl":3600}'
```

Response:

```json
{"mail_id":"mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag"}
```

Replace `mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag` with the `mail_id` returned to you in all examples below.

---

## Send Messages

mq9 supports three priority levels: `critical` (highest), `urgent`, and `normal` (default, no suffix):

```bash
# Highest priority
nats pub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.critical' '{"type":"abort","task_id":"t-001"}'

# Urgent
nats pub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.urgent' '{"type":"interrupt","task_id":"t-002"}'

# Normal (default, no suffix)
nats pub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag' '{"type":"task","payload":"process dataset A"}'
```

Sending is fire-and-forget. The sender does not wait for the recipient to be online.

---

## Subscribe and Receive

Open another terminal and subscribe to the mailbox:

```bash
# Subscribe to all priorities (critical, urgent, and normal)
nats sub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.*'
```

All messages sent above are delivered immediately in priority order: critical → urgent → normal. This is mq9's **store-first** semantics — it makes no difference whether the subscription happens before or after the messages are sent.

To subscribe to a single priority level:

```bash
nats sub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.critical'
nats sub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.urgent'
```

---

## Public Mailbox and Competing Consumers

Create a public task queue and have multiple workers compete for messages:

```bash
# Create a public mailbox
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl":3600,"public":true,"name":"task.queue"}'

# Terminal 1: Worker subscribes
nats sub '$mq9.AI.MAILBOX.MSG.task.queue.*' --queue workers

# Terminal 2: Another worker
nats sub '$mq9.AI.MAILBOX.MSG.task.queue.*' --queue workers

# Send tasks
nats pub '$mq9.AI.MAILBOX.MSG.task.queue' '{"task":"job-1"}'
nats pub '$mq9.AI.MAILBOX.MSG.task.queue' '{"task":"job-2"}'
nats pub '$mq9.AI.MAILBOX.MSG.task.queue' '{"task":"job-3"}'
```

Each task is delivered to exactly one worker.

---

## SDK Integration

Beyond the NATS CLI, mq9 supports three integration paths.

### 1. RobustMQ SDK

The RobustMQ native SDK wraps mailbox creation, publishing, and subscribing:

```bash
# Python
pip install robustmq

# JavaScript / TypeScript
npm install @robustmq/sdk

# Rust
cargo add robustmq
```

```python
from robustmq import Mq9Client

client = Mq9Client("nats://localhost:4222")

# Create a mailbox
mail_id = client.create_mailbox(ttl=3600)

# Send messages
client.publish(mail_id, {"type": "task", "payload": "run job"})
client.publish(mail_id, {"type": "abort"}, priority="critical")

# Subscribe
for msg in client.subscribe(mail_id):
    print(msg)
```

Full documentation: [SDK Integration](../mq9/SDK.md)

---

### 2. Native NATS SDK

Any NATS client library in any language works with mq9 directly — no extra dependencies:

```bash
# Python
pip install nats-py

# Go
go get github.com/nats-io/nats.go

# JavaScript
npm install nats

# Rust
cargo add async-nats
```

```python
import asyncio, json, nats

async def main():
    nc = await nats.connect("nats://localhost:4222")

    # Create a mailbox
    resp = await nc.request("$mq9.AI.MAILBOX.CREATE",
                            json.dumps({"ttl": 3600}).encode())
    mail_id = json.loads(resp.data)["mail_id"]

    # Send messages
    await nc.publish(f"$mq9.AI.MAILBOX.MSG.{mail_id}",
                     json.dumps({"type": "task"}).encode())
    await nc.publish(f"$mq9.AI.MAILBOX.MSG.{mail_id}.critical",
                     json.dumps({"type": "abort"}).encode())

    # Subscribe to all priorities
    async def handler(msg):
        print(json.loads(msg.data))
    await nc.subscribe(f"$mq9.AI.MAILBOX.MSG.{mail_id}.*", cb=handler)
    await asyncio.sleep(2)
    await nc.close()

asyncio.run(main())
```

Full examples for all languages: [SDK Integration](../mq9/SDK.md)

---

### 3. AI Framework Integration

#### LangChain / LangGraph

The `langchain-mq9` toolkit brings mq9 mailbox operations directly into LangChain Agents:

```bash
pip install langchain-mq9
```

```python
from langchain_mq9 import Mq9Toolkit
from langchain.agents import initialize_agent, AgentType
from langchain_openai import ChatOpenAI

toolkit = Mq9Toolkit(nats_url="nats://localhost:4222")
tools = toolkit.get_tools()  # create_mailbox, send_message, subscribe

agent = initialize_agent(
    tools=tools,
    llm=ChatOpenAI(model="gpt-4o"),
    agent=AgentType.OPENAI_FUNCTIONS,
)

agent.run("Create a mailbox, send a task message, and wait for the result")
```

For LangGraph, mailbox operations can be used as nodes in a workflow graph:

```python
from langchain_mq9 import Mq9Toolkit
from langgraph.graph import StateGraph

toolkit = Mq9Toolkit(nats_url="nats://localhost:4222")
# Call toolkit.send_message() / toolkit.subscribe() inside StateGraph nodes
```

Full documentation: [LangChain Integration](../mq9/LangChain.md)

#### MCP Server

mq9 ships with a standard MCP Server that exposes mailbox operations via JSON-RPC 2.0 — compatible with Dify, Claude, and other platforms without modifying existing workflows. See [MCP Server setup](../mq9/SDK.md#mcp-server).

---

## Next Steps

- **Full CLI walkthrough** — [Quick Start](../mq9/QuickStart.md)
- **Protocol reference** — [Protocol](../mq9/Protocol.md)
- **SDK integration** — [SDK](../mq9/SDK.md)
- **LangChain integration** — [LangChain](../mq9/LangChain.md)
