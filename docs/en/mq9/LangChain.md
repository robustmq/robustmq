# LangChain & LangGraph Integration

## Overview

`langchain-mq9` is an official toolkit that wraps all mq9 operations as LangChain tools. AI agents built with LangChain or LangGraph can send messages, create mailboxes, list and delete messages without any manual NATS handling.

## Installation

```
pip install langchain-mq9
```

**Requirements:** Python 3.10+, `langchain-core`, `robustmq`

---

## Available Tools

| Tool | mq9 Operation | Input | Output |
|------|--------------|-------|--------|
| `CreateMailboxTool` | Create private mailbox | `{"ttl": 3600}` | `{"mail_address": "..."}` |
| `CreatePublicMailboxTool` | Create public mailbox | `{"name": "...", "ttl": ..., "desc": "..."}` | `{"mail_address": "..."}` |
| `SendMessageTool` | Send message | `{"mail_address": "...", "content": "...", "priority": "normal"}` | ack |
| `GetMessagesTool` | Subscribe and receive messages (with payload) | `{"mail_address": "...", "limit": 10}` | messages with payload |
| `ListMessagesTool` | List message metadata (no payload) | `{"mail_address": "..."}` | metadata list |
| `DeleteMessageTool` | Delete a message | `{"mail_address": "...", "msg_id": "..."}` | `{"deleted": true}` |

> **Note:** `GetMessagesTool` fetches actual message content by subscribing briefly and collecting up to `limit` messages. `ListMessagesTool` returns only metadata (msg_id, priority, ts) without downloading payloads — use it to inspect what is in a mailbox cheaply before deciding what to retrieve or delete.

---

## Quick Start

```python
from langchain_mq9 import Mq9Toolkit

toolkit = Mq9Toolkit(server="nats://localhost:4222")
tools = toolkit.get_tools()

# Use with any LangChain agent
from langchain.agents import create_tool_calling_agent, AgentExecutor
from langchain_openai import ChatOpenAI

llm = ChatOpenAI(model="gpt-4o")
agent = create_tool_calling_agent(llm, tools, prompt)
executor = AgentExecutor(agent=agent, tools=tools)
result = executor.invoke({"input": "Create a mailbox and send me a task summary"})
```

---

## LangGraph Integration

The following example shows a two-node graph. The first node creates a mailbox and the second polls it for results. The graph loops on `check_mailbox` until at least one message arrives.

```python
from langgraph.graph import StateGraph, END
from langchain_mq9 import Mq9Toolkit
from typing import TypedDict

toolkit = Mq9Toolkit(server="nats://localhost:4222")
tools_by_name = {t.name: t for t in toolkit.get_tools()}

class State(TypedDict):
    mail_address: str
    messages: list
    done: bool

def create_mailbox(state: State) -> State:
    result = tools_by_name["create_mailbox"].run({"ttl": 3600})
    return {**state, "mail_address": result["mail_address"]}

def check_mailbox(state: State) -> State:
    msgs = tools_by_name["get_messages"].run({"mail_address": state["mail_address"], "limit": 5})
    return {**state, "messages": msgs, "done": len(msgs) > 0}

def should_continue(state: State) -> str:
    return END if state["done"] else "check_mailbox"

graph = StateGraph(State)
graph.add_node("create_mailbox", create_mailbox)
graph.add_node("check_mailbox", check_mailbox)
graph.set_entry_point("create_mailbox")
graph.add_edge("create_mailbox", "check_mailbox")
graph.add_conditional_edges("check_mailbox", should_continue)

app = graph.compile()
result = app.invoke({"mail_address": "", "messages": [], "done": False})
```

---

## Multi-Agent Communication Pattern

Two agents — an orchestrator and a worker — communicate via mq9. The orchestrator creates a private reply mailbox, sends a task to the worker's public queue, and waits for the result. The worker subscribes with a queue group so multiple worker instances share load.

```python
import asyncio
from robustmq.mq9 import Client, Priority

async def orchestrator():
    async with Client("nats://localhost:4222") as client:
        # Create reply mailbox
        reply_box = await client.create(ttl=300)

        # Send task to worker's public queue
        task = {"type": "analyze", "doc_id": "abc123", "reply_to": reply_box.mail_address}
        await client.send("task.queue", task, priority=Priority.NORMAL)

        # Wait for result
        result_event = asyncio.Event()
        result = {}

        async def on_result(msg):
            result.update(msg.data if isinstance(msg.data, dict) else {})
            result_event.set()

        sub = await client.subscribe(reply_box.mail_address, on_result)
        await asyncio.wait_for(result_event.wait(), timeout=30.0)
        await sub.unsubscribe()

        print(f"Result: {result}")

async def worker():
    async with Client("nats://localhost:4222") as client:
        # Ensure task queue exists
        await client.create(ttl=86400, public=True, name="task.queue", desc="Task queue")

        async def process(msg):
            task = msg.data
            print(f"Processing: {task}")
            # ... do work ...
            result = {"status": "done", "output": "analysis complete"}
            await client.send(task["reply_to"], result, priority=Priority.CRITICAL)

        sub = await client.subscribe("task.queue", process, queue_group="workers")
        await asyncio.sleep(60)  # run for 60 seconds
        await sub.unsubscribe()

# Run both
asyncio.run(asyncio.gather(worker(), orchestrator()))
```

Key points about this pattern:

- The orchestrator's reply mailbox has a short TTL (300 s) since it is ephemeral — one task, one reply.
- The worker's public queue has a long TTL (86400 s) and is reused across many tasks.
- `queue_group="workers"` ensures that only one worker processes each task even when multiple worker instances are running.
- `asyncio.wait_for` gives the orchestrator a hard timeout so it does not block indefinitely if the worker crashes.
