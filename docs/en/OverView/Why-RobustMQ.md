# Why RobustMQ

## The Beginning

October 2023. The first line of code was written.

The motivation was straightforward: learn Rust. Rust has a steep learning curve — you can only truly master it through real-world practice. After searching for a suitable project without success, the decision was made to build something from scratch. A message queue was the right choice: networking, concurrency, storage, and protocols — every area where Rust excels.

This project was never about commercial gain. It was about doing something **technically worth doing**.

---

## The Problems We Saw

Having worked in the messaging queue space for years, the structural problems in this field were clear — long-standing issues that had never been fundamentally solved.

**Too much historical baggage.** Kafka was born in 2011, with an architecture built on the file system. Topic counts are bounded by file descriptor limits, with tens of thousands as a practical ceiling. RabbitMQ is written in Erlang with a unique concurrency model but limited performance headroom. These systems were never designed with the AI era in mind. Later improvements were patches on top of old architectures — high complexity, low return, and increasingly difficult to keep in sync with upstream communities.

**Fragmented by scenario.** IoT uses MQTT. Big data uses Kafka. Enterprise systems use AMQP. Three protocols, three systems, three operations teams. Data flowing between systems requires bridge layers at every hop — each one adding latency and introducing failure points. No single system has ever truly unified these scenarios.

**No infrastructure designed for the AI era.** Kafka's Topic limit is a hard wall for AI Agent scenarios — 100,000 Agents each needing a few communication channels means hundreds of thousands of Topics, which brings Kafka to its knees. GPU training needs to read terabytes of data from S3, but existing systems require importing that data first — slow and wasteful. These new scenarios are not edge cases; they are the new normal. Yet no existing system addresses them at the architectural level.

These are not minor issues. They are architectural flaws. Patching them is futile. They require a clean-slate design.

---

## What We Set Out to Build

A communication infrastructure **designed from the ground up for AI, IoT, and big data** — not adapted from aging architectures.

Specifically, RobustMQ aims to:

**Simple architecture with fixed boundaries.** Three components: Meta Service for metadata, Broker for protocol handling, Storage Engine for persistence. Clear component boundaries that don't shift when new protocols or storage backends are added. Adding a new protocol means implementing parsing logic in the Broker layer. Adding new storage means implementing the Storage Engine interface. The core architecture stays stable.

**Unified multi-protocol platform.** MQTT and Kafka run on the same storage layer. Data is stored once, with each protocol providing a different access view. IoT devices write via MQTT; analytics platforms consume via Kafka — no bridge layer required.

**Million-scale Topics.** RocksDB-based KV storage frees Topic counts from file system constraints. Each AI Agent gets its own isolated communication channel. A million Agents is not a problem.

**AI-native design.** Topics connect directly to object storage (S3/MinIO). RobustMQ acts as an intelligent cache layer so GPUs don't wait for data. Shared subscriptions decouple consumer concurrency from Partition count, enabling elastic training clusters to scale freely.

**Built with Rust.** Zero-cost abstractions, memory safety, no GC pauses. Communication infrastructure demands stable latency above all else. Rust is the right language for this domain.

---

## What Kind of Project This Is

**Non-commercial.** No corporate backing, no paid version. All core features are fully open source under Apache 2.0.

**Driven by technical conviction.** We believe rebuilding communication infrastructure with Rust is the right direction. We believe the AI era deserves a messaging system designed for its actual needs, not one stretched to fit. We believe great infrastructure software should belong to the entire community. This project exists because it's worth building, not because there's a financial return.

**The goal is Apache Top-Level Project status.** Not for prestige — but because the Apache Foundation's governance model represents the best way to run an open source project: open, transparent, neutral, and sustainable. That's how we want RobustMQ to exist for the long term.

There are no shortcuts here. One step at a time. Eyes on the horizon, feet on the ground.

---

## Where We Are Now

From the first line of code in October 2023 to version 0.3.0 today: the architecture has been redesigned, MQTT core features are nearly complete, and the monitoring, CLI, and Dashboard tooling is in place. There is still much to do — Kafka protocol is under development, AI data caching is not yet complete, and performance and stability have significant room for improvement.

But the direction is clear, the architecture is solid, and the original intent has not changed.

GitHub: https://github.com/robustmq/robustmq
