# NATS Core 功能与使用

## 协议基础

NATS Core 基于 TCP 的纯文本协议，每条指令以 `\r\n`（CRLF）结尾，字段之间用空白符分隔。命令大小写不敏感，但 Subject 名称大小写敏感。

### 连接流程

```
Server → Client:  INFO {...}\r\n
Client → Server:  CONNECT {...}\r\n
Client → Server:  PING\r\n
Server → Client:  PONG\r\n
```

服务端发送 `INFO` 后，客户端回应 `CONNECT` 完成鉴权和能力声明。客户端发送 `PING`，收到 `PONG` 后连接就绪，可以开始 Pub/Sub 操作。

### Subject 命名规则

Subject 是 NATS 的寻址单元，规则如下：

- 由字母、数字、`.`、`-`、`_` 组成
- 大小写敏感：`foo.bar` 和 `Foo.Bar` 是不同的 Subject
- `.` 用于层级分隔：`orders.us.created`
- 不能以 `.` 开头或结尾
- 不能包含空格

### 通配符

| 通配符 | 说明 | 示例 |
|--------|------|------|
| `*` | 匹配单个层级 | `orders.*.created` 匹配 `orders.us.created`，不匹配 `orders.us.east.created` |
| `>` | 匹配一个或多个层级，只能放在末尾 | `orders.>` 匹配 `orders.us`、`orders.us.created`、`orders.us.east.created` |

---

## 核心命令

### PUB — 发布消息

```
PUB <subject> [reply-to] <#bytes>\r\n[payload]\r\n
```

| 参数 | 说明 |
|------|------|
| `subject` | 目标 Subject |
| `reply-to` | 可选，回复地址（用于 Request-Reply 模式） |
| `#bytes` | payload 字节数 |

示例：

```bash
# 发布到 orders.created
nats pub orders.created '{"order_id":"001","amount":100}'

# 发布并指定回复地址（手动实现 request-reply）
nats pub orders.query '{"id":"001"}' --reply orders.response.tmp
```

### SUB — 订阅

```
SUB <subject> [queue group] <sid>\r\n
```

| 参数 | 说明 |
|------|------|
| `subject` | 订阅的 Subject，支持通配符 |
| `queue group` | 可选，队列组名称（用于竞争消费） |
| `sid` | 订阅 ID，客户端自定义的订阅标识符 |

示例：

```bash
# 订阅单个 subject
nats sub orders.created

# 通配符订阅
nats sub "orders.*"
nats sub "orders.>"

# Queue Group 订阅（竞争消费）
nats sub orders.created --queue order-processors
```

### UNSUB — 取消订阅

```
UNSUB <sid> [max-msgs]\r\n
```

| 参数 | 说明 |
|------|------|
| `sid` | 要取消的订阅 ID |
| `max-msgs` | 可选，再收到 N 条消息后自动取消订阅 |

### HPUB — 发布带 Header 的消息

需要服务端支持 `headers: true`（INFO 中声明）。

```
HPUB <subject> [reply-to] <#header bytes> <#total bytes>\r\n
[headers]\r\n\r\n[payload]\r\n
```

Header 格式为 HTTP/1 风格：

```
NATS/1.0\r\n
Key1: Value1\r\n
Key2: Value2\r\n
\r\n
```

示例：

```bash
nats pub --header "Content-Type:application/json" \
         --header "X-Trace-ID:abc123" \
         orders.created '{"order_id":"001"}'
```

---

## Pub/Sub

最基础的通信模式：发布方不知道谁在订阅，订阅方不知道谁在发布。

```bash
# 终端 1：订阅
nats sub "sensor.temperature.>"

# 终端 2：发布
nats pub sensor.temperature.room1 '{"value":22.5,"unit":"celsius"}'
nats pub sensor.temperature.room2 '{"value":24.1,"unit":"celsius"}'
```

**特性：**
- 当前在线的所有订阅者都会收到消息
- 订阅者不在线时，消息直接丢失（at-most-once）
- 无需前置配置，pub 和 sub 直接使用

---

## Request-Reply

同步请求-响应模式，底层通过临时 reply-to Subject 实现。

```bash
# 服务端：监听请求并回复
nats reply orders.query '{"status":"ok","result":{"id":"001"}}'

# 客户端：发起请求，等待回复（默认超时 2 秒）
nats request orders.query '{"id":"001"}'
```

**原理：**
1. 客户端发布消息，自动生成一个临时 reply-to Subject（如 `_INBOX.abc123`）
2. 服务端收到消息后，向 reply-to Subject 发布响应
3. 客户端等待 reply-to Subject 上的消息

**Java 示例：**

```java
Connection nc = Nats.connect("nats://localhost:4222");

// 服务端（处理请求）
Dispatcher d = nc.createDispatcher((msg) -> {
    String request = new String(msg.getData());
    String response = processRequest(request);
    nc.publish(msg.getReplyTo(), response.getBytes());
});
d.subscribe("orders.query");

// 客户端（发起请求）
Message reply = nc.request("orders.query",
    "{\"id\":\"001\"}".getBytes(),
    Duration.ofSeconds(2));
System.out.println("Response: " + new String(reply.getData()));
```

---

## Queue Group（竞争消费）

多个订阅者加入同一个 Queue Group，NATS 自动将每条消息分发给其中一个订阅者，实现负载均衡。

```bash
# 启动多个 Worker（同一个 Queue Group）
nats sub orders.created --queue order-processors  # Worker 1
nats sub orders.created --queue order-processors  # Worker 2
nats sub orders.created --queue order-processors  # Worker 3

# 发布消息（只有一个 Worker 会收到）
nats pub orders.created '{"order_id":"001"}'
nats pub orders.created '{"order_id":"002"}'
nats pub orders.created '{"order_id":"003"}'
```

**特性：**
- 同一个 Queue Group 内，每条消息只投递给一个订阅者
- Worker 动态增减，无需重新配置，NATS 自动感知
- 不同 Queue Group 的订阅者相互独立，都会收到所有消息

**Java 示例：**

```java
// 三个 Worker 竞争消费
for (int i = 1; i <= 3; i++) {
    final int id = i;
    Dispatcher worker = nc.createDispatcher((msg) -> {
        System.out.println("[Worker-" + id + "] 处理: " + new String(msg.getData()));
    });
    worker.subscribe("orders.created", "order-processors");
}
```

---

## 连接与鉴权

### 连接配置

```bash
# 连接到指定地址
nats sub "test.>" --server nats://localhost:4222

# 用户名密码鉴权
nats sub "test.>" --server nats://user:password@localhost:4222

# Token 鉴权
nats sub "test.>" --server nats://mytoken@localhost:4222

# TLS
nats sub "test.>" --server nats://localhost:4222 --tlscert client.crt --tlskey client.key
```

### INFO 命令字段

服务端连接后立即推送 `INFO` JSON，完整字段如下：

| 字段 | 类型 | 说明 |
|------|------|------|
| `server_id` | string | 服务节点唯一标识符 |
| `server_name` | string | 服务节点名称 |
| `version` | string | 服务端版本号 |
| `go` | string | 编译所用语言版本 |
| `host` | string | 服务监听 IP |
| `port` | int | 服务监听端口 |
| `proto` | int | 协议版本号，`1` 表示支持动态 INFO 更新 |
| `headers` | bool | 是否支持消息 Header（HPUB/HMSG） |
| `max_payload` | int | 允许的最大 Payload 字节数 |
| `auth_required` | bool | 是否要求客户端鉴权 |
| `tls_required` | bool | 是否要求 TLS |
| `tls_verify` | bool | 是否要求客户端提供证书 |
| `tls_available` | bool | 服务端是否可选支持 TLS |
| `jetstream` | bool | 是否支持 JetStream |
| `client_id` | uint64 | 服务端分配的内部客户端 ID |
| `client_ip` | string | 客户端 IP 地址 |
| `nonce` | string | 用于 NKey 鉴权的随机数 |
| `cluster` | string | 集群名称 |
| `domain` | string | NATS 域名 |
| `connect_urls` | []string | 集群其他节点地址，用于客户端重连 |
| `ws_connect_urls` | []string | WebSocket 连接地址列表 |
| `ldm` | bool | 是否处于 Lame Duck 模式（即将下线） |
| `git_commit` | string | 编译版本的 Git commit hash |
| `cluster_dynamic` | bool | 集群是否支持动态路由，仅在配置了集群路由时存在 |
| `xkey` | string | 服务端 X25519 公钥，用于消息级加密（NKey 加密扩展），普通部署不使用 |

### CONNECT 命令字段

客户端连接时向服务端发送 `CONNECT` JSON，常用字段：

| 字段 | 类型 | 说明 |
|------|------|------|
| `verbose` | bool | 是否为每条命令返回 `+OK` 确认 |
| `pedantic` | bool | 是否启用严格模式（校验 Subject 合法性等） |
| `tls_required` | bool | 是否要求 TLS |
| `name` | string | 客户端名称，方便调试 |
| `lang` | string | 客户端语言，如 `go`、`java`、`python` |
| `version` | string | 客户端版本 |
| `protocol` | int | 协议版本，`1` 表示支持动态 INFO 更新 |
| `user` | string | 用户名（用户名密码鉴权） |
| `pass` | string | 密码 |
| `auth_token` | string | Token 鉴权 |
| `echo` | bool | 是否将自己发布的消息回传给自己的订阅，默认 `true` |
| `sig` | string | 对 `nonce` 的签名，NKey 鉴权时使用 |
| `jwt` | string | 用户 JWT，用于权限控制 |
| `nkey` | string | NKey 公钥 |
| `no_responders` | bool | 是否启用无订阅者时立即返回错误的功能 |
| `headers` | bool | 是否支持消息 Header |

---

## 心跳与保活

NATS 通过 PING/PONG 保持连接活跃。服务端会定期向客户端发送 `PING`，客户端必须回复 `PONG`。客户端也可以主动发 `PING` 来探测连接是否存活。

```
Server → Client: PING\r\n
Client → Server: PONG\r\n
```

如果客户端在规定时间内没有回复 `PONG`，服务端会关闭连接。客户端 SDK 通常会自动处理 PING/PONG，不需要手动管理。

---

## 错误处理

服务端返回的错误格式：

```
-ERR '<error message>'\r\n
```

常见错误：

| 错误 | 说明 |
|------|------|
| `'Unknown Protocol Operation'` | 收到无法识别的命令 |
| `'Attempted To Connect To Route Port'` | 客户端连接了集群路由端口 |
| `'Authorization Violation'` | 鉴权失败 |
| `'Authorization Timeout'` | 鉴权超时 |
| `'Invalid Client Protocol'` | 协议版本不兼容 |
| `'Maximum Control Line Exceeded'` | 控制行超过最大长度 |
| `'Parser Error'` | 协议解析错误 |
| `'Secure Connection - TLS Required'` | 需要 TLS 连接 |
| `'Stale Connection'` | 连接已过期（PING/PONG 超时） |
| `'Maximum Connections Exceeded'` | 达到最大连接数 |
| `'Slow Consumer'` | 消费者处理消息过慢，缓冲区溢出 |
| `'Maximum Payload Violation'` | 消息体超过最大限制 |
| `'Invalid Subject'` | Subject 格式非法 |
| `'Permissions Violation'` | 权限不足（订阅或发布） |

---

## 与 mq9 的关系

NATS Core 是 mq9 的底层协议。mq9 在 NATS Core 的 pub/sub/req-reply 基础上，通过 Subject 命名约定（`$mq9.AI.*`）增加了持久化、优先级和 TTL 管理，专为 AI Agent 异步通信设计。

两者可以混用：普通 NATS pub/sub 用于实时场景，mq9 Subject 用于需要离线保障的 Agent 通信。

详见 [mq9 协议设计](/zh/mq9/Protocol)。
