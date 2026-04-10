# SDK 接入

RobustMQ 完整兼容 NATS 协议，使用官方 NATS 客户端 SDK 直接连接，无需任何修改。

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
