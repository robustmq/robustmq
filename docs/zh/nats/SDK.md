# SDK 接入

RobustMQ 支持本文档列出的 NATS Core 指令子集，并使用标准 NATS 线协议。官方 NATS 客户端 SDK 可以直接连接并使用已支持的操作，但 JetStream 以及 Token/JWT/NKey 鉴权等子集之外的能力尚不可用。

## 各语言 SDK

| 语言 | 安装 | 官方文档 |
|------|------|---------|
| Go | `go get github.com/nats-io/nats.go` | [nats.go](https://github.com/nats-io/nats.go) |
| Python | `pip install nats-py` | [nats.py](https://github.com/nats-io/nats.py) |
| JavaScript / Node.js | `npm install nats` | [nats.js](https://github.com/nats-io/nats.js) |
| Java | Maven / Gradle | [jnats](https://github.com/nats-io/jnats) |
| Rust | `cargo add async-nats` | [nats.rs](https://github.com/nats-io/nats.rs) |
| C# / .NET | `dotnet add package NATS.Net` | [nats.net](https://github.com/nats-io/nats.net) |
| C | 源码编译 | [nats.c](https://github.com/nats-io/nats.c) |
| Ruby | `gem install nats-pure` | [nats-pure.rb](https://github.com/nats-io/nats-pure.rb) |

完整 SDK 列表及示例：[nats.io/download](https://nats.io/download/)

## 连接地址

将官方文档中的连接地址替换为 RobustMQ 的地址即可：

```text
nats://localhost:4222
```

其余代码无需任何改动。
