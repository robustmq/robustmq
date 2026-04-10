# 体验 mq9

## 前提：启动 Broker

参考 [快速安装](Quick-Install.md) 完成安装，然后启动服务：

```bash
robust-server start
```

启动成功后验证状态：

```bash
robust-ctl status
```

mq9 随 RobustMQ 启动，无需额外配置，默认监听 NATS 端口 `4222`。

---

## 准备 NATS CLI

mq9 基于 NATS 协议，只需安装 NATS CLI 即可体验所有操作：

```bash
# macOS
brew install nats-io/nats-tools/nats

# Linux / Windows
# 参考：https://docs.nats.io/using-nats/nats-tools/nats_cli
```

安装完成后设置连接地址：

```bash
export NATS_URL=nats://localhost:4222
```

---

## 创建邮箱

邮箱是 mq9 的基本通信地址。使用 `nats req` 创建，服务端返回分配的 `mail_id`：

```bash
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl":3600}'
```

响应：

```json
{"mail_id":"mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag"}
```

将下面示例中的 `mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag` 替换为你实际拿到的 `mail_id`。

---

## 发送消息

mq9 支持三个优先级：`critical`（最高）、`urgent`（紧急）、`normal`（默认，无后缀）：

```bash
# 最高优先级
nats pub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.critical' '{"type":"abort","task_id":"t-001"}'

# 紧急
nats pub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.urgent' '{"type":"interrupt","task_id":"t-002"}'

# 默认优先级（normal，无后缀）
nats pub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag' '{"type":"task","payload":"process dataset A"}'
```

消息即发即忘，发送方无需等待接收方在线。

---

## 订阅接收消息

打开另一个终端，订阅邮箱：

```bash
# 订阅所有优先级（critical、urgent、normal）
nats sub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.*'
```

订阅后立即收到上面发送的所有消息，顺序为 critical → urgent → normal。这是 mq9 的**先存储后推送**语义——无论订阅发生在消息前还是后，结果一样。

只订阅某一优先级：

```bash
nats sub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.critical'
nats sub '$mq9.AI.MAILBOX.MSG.mail-d7a5072lko83gp7amga0-d7a5072lko83gp7amgag.urgent'
```

---

## 公开邮箱与竞争消费

创建一个公开任务队列，多个 Worker 竞争消费：

```bash
# 创建公开邮箱
nats req '$mq9.AI.MAILBOX.CREATE' '{"ttl":3600,"public":true,"name":"task.queue"}'

# 终端 1：Worker 订阅
nats sub '$mq9.AI.MAILBOX.MSG.task.queue.*' --queue workers

# 终端 2：另一个 Worker
nats sub '$mq9.AI.MAILBOX.MSG.task.queue.*' --queue workers

# 发送任务
nats pub '$mq9.AI.MAILBOX.MSG.task.queue' '{"task":"job-1"}'
nats pub '$mq9.AI.MAILBOX.MSG.task.queue' '{"task":"job-2"}'
nats pub '$mq9.AI.MAILBOX.MSG.task.queue' '{"task":"job-3"}'
```

每条任务只会被一个 Worker 收到。

---

## SDK 接入

除 NATS CLI 外，mq9 支持三种接入方式。

### 1. RobustMQ SDK

RobustMQ 提供原生 mq9 SDK，封装了邮箱创建、发送、订阅等操作：

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

# 创建邮箱
mail_id = client.create_mailbox(ttl=3600)

# 发送消息
client.publish(mail_id, {"type": "task", "payload": "run job"})
client.publish(mail_id, {"type": "abort"}, priority="critical")

# 订阅
for msg in client.subscribe(mail_id):
    print(msg)
```

完整文档：[SDK 接入](../mq9/SDK.md)

---

### 2. NATS 原生 SDK

任何语言的 NATS 客户端库都可以直接操作 mq9，无需额外依赖：

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

    # 创建邮箱
    resp = await nc.request("$mq9.AI.MAILBOX.CREATE",
                            json.dumps({"ttl": 3600}).encode())
    mail_id = json.loads(resp.data)["mail_id"]

    # 发送消息
    await nc.publish(f"$mq9.AI.MAILBOX.MSG.{mail_id}",
                     json.dumps({"type": "task"}).encode())
    await nc.publish(f"$mq9.AI.MAILBOX.MSG.{mail_id}.critical",
                     json.dumps({"type": "abort"}).encode())

    # 订阅所有优先级
    async def handler(msg):
        print(json.loads(msg.data))
    await nc.subscribe(f"$mq9.AI.MAILBOX.MSG.{mail_id}.*", cb=handler)
    await asyncio.sleep(2)
    await nc.close()

asyncio.run(main())
```

各语言完整示例：[NATS 客户端接入](../mq9/NatsClient.md)

---

### 3. AI 框架集成

#### LangChain / LangGraph

通过 `langchain-mq9` 工具包，将 mq9 邮箱能力直接注入 LangChain Agent：

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

agent.run("创建一个邮箱，发送任务消息，然后等待结果")
```

LangGraph 中可将邮箱操作作为节点接入工作流：

```python
from langchain_mq9 import Mq9Toolkit
from langgraph.graph import StateGraph

toolkit = Mq9Toolkit(nats_url="nats://localhost:4222")
# 在 StateGraph 节点中调用 toolkit.send_message() / toolkit.subscribe()
```

完整文档：[LangChain 集成](../mq9/LangChain.md)

#### MCP Server

mq9 提供标准 MCP Server，供 Dify、Claude 等平台通过 JSON-RPC 2.0 调用邮箱能力，无需修改现有工作流。配置方式参考 [MCP Server 文档](../mq9/SDK.md#mcp-server)。

---

## 下一步

- **完整 CLI 演练** — [快速开始](../mq9/QuickStart.md)
- **协议设计** — [协议设计](../mq9/Protocol.md)
- **SDK 接入** — [SDK 接入](../mq9/SDK.md)
- **LangChain 集成** — [LangChain 集成](../mq9/LangChain.md)
