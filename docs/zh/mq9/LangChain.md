# LangChain & LangGraph 集成

## 概述

`langchain-mq9` 是官方工具包，将所有 mq9 操作封装为 LangChain 工具。使用 LangChain 或 LangGraph 构建的 AI Agent 可以直接发送消息、创建邮箱、列出和删除消息，无需手动处理 NATS。

## 安装

```
pip install langchain-mq9
```

**依赖：** Python 3.10+、`langchain-core`、`robustmq`

---

## 可用工具

| 工具 | mq9 操作 | 输入 | 输出 |
|------|---------|------|------|
| `CreateMailboxTool` | 创建私有邮箱 | `{"ttl": 3600}` | `{"mail_address": "..."}` |
| `CreatePublicMailboxTool` | 创建公开邮箱 | `{"name": "...", "ttl": ..., "desc": "..."}` | `{"mail_address": "..."}` |
| `SendMessageTool` | 发送消息 | `{"mail_address": "...", "content": "...", "priority": "normal"}` | ack |
| `GetMessagesTool` | 订阅并接收消息（含消息体） | `{"mail_address": "...", "limit": 10}` | 含消息体的消息列表 |
| `ListMessagesTool` | 列出消息元数据（不含消息体） | `{"mail_address": "..."}` | 元数据列表 |
| `DeleteMessageTool` | 删除消息 | `{"mail_address": "...", "msg_id": "..."}` | `{"deleted": true}` |

> **说明：** `GetMessagesTool` 通过短暂订阅收集最多 `limit` 条消息，返回实际消息内容。`ListMessagesTool` 只返回元数据（msg_id、priority、ts），不下载消息体——用它可以廉价地检视邮箱内容，再决定检索或删除哪些消息。

---

## 快速开始

```python
from langchain_mq9 import Mq9Toolkit

toolkit = Mq9Toolkit(server="nats://localhost:4222")
tools = toolkit.get_tools()

# 与任意 LangChain Agent 配合使用
from langchain.agents import create_tool_calling_agent, AgentExecutor
from langchain_openai import ChatOpenAI

llm = ChatOpenAI(model="gpt-4o")
agent = create_tool_calling_agent(llm, tools, prompt)
executor = AgentExecutor(agent=agent, tools=tools)
result = executor.invoke({"input": "创建一个邮箱并发送任务摘要"})
```

---

## LangGraph 集成

以下示例展示一个双节点图：第一个节点创建邮箱，第二个轮询邮箱等待结果。图在 `check_mailbox` 上循环，直到收到至少一条消息。

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

## 多 Agent 通信模式

两个 Agent——编排者和 Worker——通过 mq9 通信。编排者创建一个私有回复邮箱，将任务发送到 Worker 的公开队列，然后等待结果。Worker 以队列组方式订阅，确保多个 Worker 实例共享负载。

```python
import asyncio
from robustmq.mq9 import Client, Priority

async def orchestrator():
    async with Client("nats://localhost:4222") as client:
        # 创建回复邮箱
        reply_box = await client.create(ttl=300)

        # 将任务发送到 Worker 的公开队列
        task = {"type": "analyze", "doc_id": "abc123", "reply_to": reply_box.mail_address}
        await client.send("task.queue@mq9", task, priority=Priority.NORMAL)

        # 等待结果
        result_event = asyncio.Event()
        result = {}

        async def on_result(msg):
            result.update(msg.data if isinstance(msg.data, dict) else {})
            result_event.set()

        sub = await client.subscribe(reply_box.mail_address, on_result)
        await asyncio.wait_for(result_event.wait(), timeout=30.0)
        await sub.unsubscribe()

        print(f"结果: {result}")

async def worker():
    async with Client("nats://localhost:4222") as client:
        # 确保任务队列存在
        await client.create(ttl=86400, public=True, name="task.queue@mq9", desc="任务队列")

        async def process(msg):
            task = msg.data
            print(f"处理中: {task}")
            # ... 执行工作 ...
            result = {"status": "done", "output": "分析完成"}
            await client.send(task["reply_to"], result, priority=Priority.CRITICAL)

        sub = await client.subscribe("task.queue@mq9", process, queue_group="workers")
        await asyncio.sleep(60)  # 运行 60 秒
        await sub.unsubscribe()

# 同时运行两者
asyncio.run(asyncio.gather(worker(), orchestrator()))
```

此模式的关键点：

- 编排者的回复邮箱 TTL 短（300 秒），因为它是临时的——一个任务，一个回复。
- Worker 的公开队列 TTL 长（86400 秒），跨多个任务复用。
- `queue_group="workers"` 确保即使有多个 Worker 实例运行，每条任务也只被一个 Worker 处理。
- `asyncio.wait_for` 给编排者设置硬超时，防止 Worker 崩溃时无限阻塞。
