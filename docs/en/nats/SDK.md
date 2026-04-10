# SDK Integration

RobustMQ is fully compatible with the NATS protocol. Use any official NATS client SDK to connect directly — no changes required.

## SDKs by Language

| Language | Install | Official Docs |
|----------|---------|---------------|
| Go | `go get github.com/nats-io/nats.go` | [nats.go](https://github.com/nats-io/nats.go) |
| Python | `pip install nats-py` | [nats.py](https://github.com/nats-io/nats.py) |
| JavaScript / Node.js | `npm install nats` | [nats.js](https://github.com/nats-io/nats.js) |
| Java | Maven / Gradle | [jnats](https://github.com/nats-io/jnats) |
| Rust | `cargo add async-nats` | [nats.rs](https://github.com/nats-io/nats.rs) |
| C# / .NET | `dotnet add package NATS.Net` | [nats.net](https://github.com/nats-io/nats.net) |
| C | build from source | [nats.c](https://github.com/nats-io/nats.c) |
| Ruby | `gem install nats-pure` | [nats-pure.rb](https://github.com/nats-io/nats-pure.rb) |

Full SDK list and examples: [nats.io/download](https://nats.io/download/)

## Connection URL

Replace the server address in the official docs with your RobustMQ address:

```text
nats://localhost:4222
```

Everything else works as-is — no code changes needed.
