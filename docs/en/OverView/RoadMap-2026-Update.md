# 2026 RoadMap (Updated)

> This page is a revision of the [2026 RoadMap](./RoadMap-2026.md). The old plan is kept as a historical snapshot and is no longer updated; this page is the currently active plan.

## Why the Plan Changed

The original roadmap was written from the vantage point of **v0.3.0**: a three-phase plan for the year — first get MQTT production-ready (v0.4.0), then bring Kafka to basic read/write functionality (v0.5.0), then explore AI MQ in parallel (v0.6.0). AMQP and RocketMQ were explicitly marked as "not in scope for 2026," and NATS wasn't mentioned at all.

Actual progress has exceeded that plan: **the unified "one data copy, multi-protocol native views" foundation has been validated end to end** — MQTT, Kafka, NATS, mq9, and AMQP all have first versions running on the same foundation, with core logic complete. This isn't "slightly ahead of schedule" — it's architectural validation for things the old plan explicitly said were out of scope for 2026, already done.

Because the foundation has now been proven, **there's no need to keep pushing all five protocols forward at once** — that would only spread effort thin and delay each protocol from actually reaching production readiness. So the plan for the second half of the year is to **narrow the scope back down**: take the "MQTT writes, Kafka consumes" path that already works end to end and make it solid and production-ready, with a concentrated focus on stability and performance rather than continuing to expand protocol breadth.

## H2 2026 Core Focus: The MQTT + Kafka Line

### 1. MQTT Production Readiness

- Cluster stability: long-running validation and self-healing after node crash/restart
- Performance baselines: publishable benchmarks for connection count, pub/sub throughput, and P99 latency
- Observability: metrics/dashboard coverage for real production operations
- Security: hardening the stability and performance of TLS/mTLS, ACL, and authentication paths

### 2. Kafka Production Readiness

- Validate the core read/write path (Produce/Fetch/Offset/Consumer Group) for stability under real workloads
- Continued compatibility testing against mainstream Kafka clients (Java Client, official CLI tools), fixing edge cases
- Performance baselines: publishable throughput and latency benchmarks

### 3. Making the MQTT ↔ Kafka Path Solid

The unified storage layer has already proven that "publish over MQTT, consume the same data over Kafka" works (see the [homepage demo](/en/) and the IoT scenario docs). For H2, the goal is to take it from "works" to "production-ready and stress-tested":

- Stress testing and edge-case fixes for cross-protocol consistency scenarios
- Resource isolation and performance validation under combined high-concurrency MQTT writes and high-throughput Kafka consumption
- Filling in operational observability for this path (full visibility from an MQTT write to Kafka consumption)

### 4. Remote Storage — S3 Adaptation

- Automatic cold-data tiering to S3 / S3-compatible object storage, reducing long-term storage cost
- Production-grade validation of the tiering strategy: read/write path stability and performance under mixed hot/cold access

### Other Protocols: Validated, Not the Current Focus

The first versions of NATS, AMQP, and mq9 already run on the same foundation, proving the architecture's extensibility — but **they are not the H2 investment focus**. They'll stay in a "usable, lightly maintained" state until the MQTT + Kafka line reaches production readiness, at which point investment priority will be reassessed.

## Release Cadence

The timing of major-version milestones stays as originally planned:

| Version | Target | Content (updated to reflect the new focus) |
|---------|--------|---------------------------------------------|
| v0.4.x | Released, iterating | Unified foundation validated: first versions of MQTT/Kafka/NATS/mq9/AMQP all running |
| v0.5.0 | Target: September 2026 | MQTT production-readiness milestone, Kafka production-readiness milestone, MQTT↔Kafka path hardened, initial S3 remote storage support |
| v0.6.0 | Target: December 2026 | MQTT + Kafka production-ready, performance benchmark report published, S3 remote storage production-ready |

Unlike the old plan: **there will be more minor releases between major versions**, continuously shipping incremental stability and performance improvements instead of batching everything up for the major-version milestone — a rhythm that's already visible in the 0.4.x series (multiple point releases shipped since v0.4.0).

## Further Reading

- [2026 RoadMap (original, historical snapshot)](./RoadMap-2026.md)
- [RobustMQ vs. Existing MQ Systems](./Summary.md)
- [The Philosophy RobustMQ Follows](/en/Blogs/83)
