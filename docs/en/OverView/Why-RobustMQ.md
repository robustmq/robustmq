# Why RobustMQ

## The Beginning

October 2023. The first line of code was written.

The motivation was simple: learn Rust. Rust has a steep learning curve — you can only truly master it through real-world practice. After searching for a suitable project without success, the decision was made to build a message queue from scratch. Networking, concurrency, storage, and protocols — every area where Rust excels. This project was never about commercial gain. It was about doing something **technically worth doing**.

---

## The Problems We Saw

Having worked in the messaging space for years, the structural problems in this field were clear — long-standing issues that had never been fundamentally solved.

**Too much historical baggage.** Kafka was born in 2011, with an architecture built on the file system. Topic counts are bounded by file descriptor limits. RabbitMQ is written in Erlang with a unique concurrency model but limited performance headroom. These systems were never designed with the AI era in mind. Later improvements are patches on old foundations — high complexity, low return, and increasingly hard to keep in sync with upstream communities.

**Fragmented by scenario.** IoT uses MQTT brokers, data pipelines use Kafka, enterprise systems use RabbitMQ, and AI Agent communication has no native solution. Multiple protocols, multiple systems, multiple operations teams. Data flowing between systems requires bridge layers at every hop — each one adding latency and introducing failure points. No single system has ever truly unified these scenarios.

**High deployment barrier, limited coverage.** Existing messaging systems universally depend on external components — Kafka requires ZooKeeper or KRaft, RabbitMQ requires an Erlang runtime, RocketMQ requires a separately deployed NameServer. Starting a system means two to six processes, with memory footprints starting in the hundreds of megabytes. This is acceptable on cloud servers, but impossible on edge gateways and resource-constrained IoT devices. Operational complexity and learning overhead remain persistently high. A messaging system should be infrastructure — not itself become a complex system requiring dedicated maintenance.

**No infrastructure designed for the AI era.** Large-scale AI Agent collaboration requires millions of independent communication channels. Existing systems hit hard Topic limits. Edge-to-cloud data paths need cross-protocol interoperability — current solutions either stitch together multiple systems or rely on routing engines, requiring multiple data copies, increasing both latency and cost. These new scenarios are not addressed at the architectural level by any existing system.

These are not minor issues. They are architectural flaws. Patching them is futile. They require a clean-slate design.

---

## What We Set Out to Build

A communication infrastructure **designed from the ground up for the AI era** — not adapted from aging architectures.

Specifically, RobustMQ aims to:

**Simple architecture with fixed boundaries.** Three components: Meta Service for metadata, Broker for protocol handling, Storage Engine for persistence. Clear component boundaries that do not shift when new protocols or storage backends are added. Adding a new protocol means implementing parsing logic in the Broker layer only. Adding new storage means implementing the Storage Engine interface only. The core architecture stays stable.

**Unified multi-protocol platform, one copy of data.** MQTT, Kafka, NATS, and AMQP run on the same storage layer. Data is stored once, with each protocol providing a different access view. IoT devices write via MQTT, analytics platforms consume via Kafka, AI Agents communicate via NATS, enterprise systems connect via AMQP — no bridge layer required. One message, consumed by any protocol.

**Edge-to-cloud unified deployment.** Single binary, no external dependencies, minimal memory footprint. The same system runs on edge gateways for offline buffering and on cloud clusters for large-scale data flow — consistent operations from edge to cloud.

**AI Agent communication, natively.** The `$AI.API.*` subject space, built on the NATS protocol, provides native capabilities for Agent registration, discovery, invocation, and orchestration. Millions of lightweight Topics — each Agent gets its own isolated communication channel, unconstrained by traditional Topic limits.

**Built with Rust.** Zero-cost abstractions, memory safety, no GC pauses. Communication infrastructure demands stable latency above all else. GC-induced memory spikes are especially damaging in edge scenarios. Rust is the right language for this domain.

---

## How We Build It

Slow is smooth, smooth is fast. Focused and disciplined.

First: bring MQTT to excellence — become the best MQTT Broker available, using that process to harden the architecture, the code, and the engineering culture. Second: explore NATS and AI Agent communication, letting AI scenarios define the future of messaging. Third: advance Kafka compatibility, completing the IoT-to-streaming pipeline and making edge-to-cloud data flow a reality. AMQP comes last.

No distractions, no chasing trends. Every step done properly.

---

## What Kind of Project This Is

**Driven by technical conviction.** We believe rebuilding communication infrastructure with Rust is the right direction. We believe the AI era needs a messaging system truly designed for its actual needs, not one stretched to fit. We believe excellent infrastructure software should belong to the entire community. This project exists because it is worth building, not because of financial return.

There are no shortcuts here. One step at a time. Eyes on the horizon, feet on the ground.

---

## Where We Are Now

From the first line of code in October 2023 to version 0.3.0 today: the architecture has been redesigned, MQTT core features are nearly complete, and monitoring, CLI, and Dashboard tooling is in place. Kafka protocol is under active development, NATS and AMQP demos are complete. Performance and stability have significant room for improvement.

The direction is clear, the architecture is solid, and the original intent has not changed.

GitHub: https://github.com/robustmq/robustmq
