# Experience NATS Core

## Prerequisites: Start the Broker

Follow [Quick Install](Quick-Install.md) to install RobustMQ, then start the service:

```bash
robust-server start
```

RobustMQ starts NATS on port `4222` by default — no extra configuration needed.

---

## Install NATS CLI

```bash
# macOS
brew install nats-io/nats-tools/nats

# Linux / Windows
# See: https://docs.nats.io/using-nats/nats-tools/nats_cli
```

Set the server URL (optional, avoids typing `--server` each time):

```bash
export NATS_URL=nats://localhost:4222
```

Verify the connection:

```bash
nats server ping
```

---

## Pub/Sub

The simplest publish-subscribe pattern.

```bash
# Terminal 1: subscribe
nats sub "hello.>"

# Terminal 2: publish
nats pub hello.world "Hello RobustMQ!"
nats pub hello.nats  "NATS is working!"
```

Terminal 1 receives both messages immediately. `>` is a wildcard that matches all tokens under `hello.`.

---

## Request-Reply

Synchronous request-response — ideal for RPC scenarios.

```bash
# Terminal 1: start a service that replies to requests
nats reply orders.query '{"status":"ok"}'

# Terminal 2: send a request and wait for the reply
nats request orders.query '{"id":"001"}'
```

---

## Queue Groups (competing consumers)

Multiple consumers join the same Queue Group. Each message is delivered to exactly one of them, providing load balancing.

```bash
# Terminals 1, 2, 3: start three workers in the same Queue Group
nats sub orders.created --queue order-processors   # Worker 1
nats sub orders.created --queue order-processors   # Worker 2
nats sub orders.created --queue order-processors   # Worker 3

# Terminal 4: publish messages (each goes to exactly one worker)
nats pub orders.created '{"order_id":"001"}'
nats pub orders.created '{"order_id":"002"}'
nats pub orders.created '{"order_id":"003"}'
```

---

## Messages with Headers

```bash
nats pub --header "Content-Type:application/json" \
         --header "X-Trace-ID:abc123" \
         orders.created '{"order_id":"001"}'
```

---

## Next Steps

- [NATS Core Reference](../nats/NatsCore.md) — protocol details, wildcards, auth, TLS, and more
- [SDK Integration](../nats/SDK.md) — client library links for all languages
- [Experience mq9](Experience-MQ9.md) — AI Agent communication protocol built on NATS
