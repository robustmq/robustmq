# RoadMap 2026

## Overview

The current stable release is **v0.3.0**. In 2026, RobustMQ plans to release **2–3 major versions**, starting from v0.4.0 and advancing to v0.5.0 or v0.6.0.

The overarching theme for the year is **code quality, performance, and stability**, with three key milestones:

| Milestone | Goal |
|-----------|------|
| MQTT Broker — Production Ready | Stable cluster operation, passing stress benchmarks, complete observability and ops tooling |
| Kafka — Basic Functionality | Core read/write path functional; mainstream Kafka clients connect without code changes |
| AI MQ — Initial Exploration | Object storage data source, intelligent cache layer, and AI Agent communication channels available in preview |

---

## v0.4.0 (Target: May 2026)

> Goal: MQTT Broker reaches production-ready standards. Full-scale code refactoring and performance tuning.

### MQTT Broker

#### Performance & Stability
1. Complete a comprehensive code refactor to eliminate technical debt
2. Stress-test MQTT cluster mode (connections, pub/sub throughput, latency) and establish baselines
3. Fix all blocking bugs discovered during stress testing
4. Connection layer optimization: reduce unnecessary memory allocations, lower P99 latency
5. Message routing optimization: improve shared subscription dispatch throughput

#### Reliability
1. Harden the QoS 1/2 acknowledgment chain; fix edge cases
2. Session persistence (RocksDB) — reliability validation and test coverage
3. Seamless session recovery after cluster node crash or restart

#### Observability
1. Complete Prometheus metrics coverage across connections, publish, subscribe, and storage dimensions
2. Grafana Dashboard improvements with ready-to-use monitoring templates
3. Structured logging: unified log format with filtering by ClientID / Topic

#### Security
1. Stabilize TLS/mTLS mutual authentication
2. ACL rule engine performance optimization
3. JWT authentication support

#### Testing
1. Unit test coverage target: ≥ 60%
2. Integration tests: full MQTT 3.x / 5.0 protocol flow coverage
3. Chaos testing: behavior validation under node failure and network partition scenarios

### Meta Service
1. Multi Raft election and log replication stability verification
2. Stress-test Meta Service gRPC interfaces and optimize hot paths
3. Improve snapshot and log compaction mechanisms

### Storage Engine (Journal)
1. Cluster mode (multi-replica ISR) stable operation
2. Segment file management: expiration cleanup, capacity alerting
3. Journal Client reconnect and retry mechanism improvements

### Docs & Community
1. Improve production deployment docs (K8s, Docker Compose cluster mode)
2. Publish a performance benchmark report
3. Add contributor guide to lower the barrier for community participation

---

## v0.5.0 (Target: September 2026)

> Goal: Kafka basic functionality available. AI MQ initial shape. Continued MQTT enhancements.

### Kafka Basic Functionality
1. **Producer protocol**: Parse Produce requests, write messages to Journal Engine
2. **Consumer protocol**: Fetch requests, Offset management, basic Consumer Group support
3. **Topic management**: CreateTopics / DeleteTopics / DescribeTopics
4. **Compatibility testing**: Java Kafka Client, kafka-console-producer/consumer direct connection
5. Admin API support for Kafka Topic query and management

### AI MQ (Exploration Phase)
1. **Object storage data source**: Topics can directly mount S3 / MinIO paths and support sequential reads
2. **Intelligent cache layer (prototype)**: Three-tier cache framework (memory → SSD → object storage)
3. **AI Agent Channels**: Best practices and examples for isolated inter-Agent communication via MQTT shared subscriptions
4. Publish AI MQ technical design document and collect community feedback

### MQTT Broker (Capability Enhancement)
1. Rule engine: enhanced SQL filter capabilities, more downstream targets (HTTP Webhook, Kafka, GreptimeDB)
2. Data bridges: improve stability of Kafka, MySQL, and PostgreSQL bridges
3. Slow subscription detection and alerting
4. Fine-grained rate limiting for connections, message rate, and traffic
5. MQTT over QUIC (early research)

### Operations Tooling
1. CLI: cluster state diagnostics, connection query, Topic statistics
2. HTTP Admin API: fill in missing endpoints, provide complete OpenAPI documentation
3. Dashboard: alert configuration, rule engine visual configuration

---

## v0.6.0 (Target: December 2026)

> Goal: Kafka functionality continues to mature. AI MQ feature enhancements.

### Kafka — Continued Improvements
1. Full Consumer Group Rebalance implementation
2. Transactional Producer basic support
3. Kafka Streams and Kafka Connect compatibility testing

### AI MQ (Feature Enhancement)
1. Intelligent prefetching: predict access patterns and preload data from object storage into memory/SSD
2. Multi-tenant AI Channels: Namespace-based isolation supporting concurrent AI training workloads
3. GPU-friendly batch read interface (large batch, low latency)

### MQTT Broker (Ecosystem)
1. MQTT Bridge support for additional protocols (Redis Streams, RabbitMQ)
2. Plugin extension framework (custom authentication, custom routing rules)
3. Multi-datacenter / geo-redundancy solution (early research)

---

## Long-term Protocol Roadmap (Post-2026)

RobustMQ's architecture has supported multi-protocol extensibility from day one — protocol handling in the Broker layer is pluggable by design. The following protocols are part of the long-term plan, but **none are scheduled for 2026**. The current focus is on doing MQTT and Kafka well first:

| Protocol | Status | Notes |
|----------|--------|-------|
| MQTT 3.x / 5.0 | ✅ Available | Core protocol, continuously improved |
| Kafka | 🚧 In development | Taking shape progressively through 2026 |
| AMQP | 📅 Long-term plan | Architecturally supported, no short-term schedule |
| RocketMQ | 📅 Long-term plan | Architecturally supported, no short-term schedule |

---

## Year-round Objectives

The following goals apply across all versions throughout the year:

| Category | Goal |
|----------|------|
| Code Quality | One dedicated code review and refactor pass per version; eliminate all Clippy warnings |
| Testing | Continuously increase unit test and integration test coverage |
| Documentation | Every feature ships with Chinese and English docs, updated simultaneously |
| Community | Quarterly RobustMQ progress blog posts; maintain an active Good First Issue list |
| Release Cadence | Each major version ships with binary packages + multi-arch Docker images via automated CI/CD |

---

## Version Timeline

```
May 2026       ──►  v0.4.0  MQTT Production Ready
September 2026 ──►  v0.5.0  Kafka basic functionality + AI MQ initial shape
December 2026  ──►  v0.6.0  Kafka continued improvements + AI MQ enhancements
```
