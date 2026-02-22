# Storage Engine Architecture

The storage engine is the core of a message queue — it determines the performance ceiling, cost floor, and range of scenarios a system can serve. RobustMQ uses a pluggable storage design, providing three engines (Memory, RocksDB, File Segment) that are abstracted from the upper-layer Broker through a unified Storage Adapter. This allows the same cluster to flexibly select a storage strategy at the Topic level.

---

## Why Pluggable Storage

No single storage engine can cover all message queue scenarios. The typical use cases roughly fall into five categories:

| Scenario | Representative | Core Requirements |
|----------|---------------|-------------------|
| Low latency, high throughput | Kafka | Persistent, multi-replica, high throughput, millisecond latency |
| High throughput, low cost | Pulsar (tiered storage) | Large data volumes, object storage for cost reduction |
| Million-scale Topics/Partitions | RocketMQ | Massive partitions, data isolation, not optimizing for peak throughput |
| Ultra-low latency, high QPS | NATS | Tolerates data loss, microsecond latency, pure in-memory |
| Edge lightweight deployment | No standard solution | Zero external dependencies, lightweight, resource-constrained |

The storage requirements of these five scenarios differ enormously, and a single engine will inevitably make compromises. RobustMQ's answer is pluggable storage: **one architecture, covering everything from edge to cloud, from memory to object storage, through configuration alone**.

Pluggability isn't about "supporting many storage systems" — it's about leaving enough architectural room for extension. When a business encounters an unanticipated scenario, they can implement their own Storage Adapter instead of forking the code.

---

## Three Storage Engines

### Memory

Pure in-memory + DashMap implementation, supporting four index types: offset, tag, key, and timestamp. Data is lost when the process restarts.

**Target scenarios**: Real-time market data push, sensor sampling, game temporary state — scenarios that tolerate some data loss but are extremely latency-sensitive.

**Replica strategies**:
- Single replica: 100µs latency, all data lost on node restart, for extreme performance scenarios
- Dual replica (acks=1): Leader returns immediately on write, async replication to the second replica in the background; 100-200µs latency with significantly better reliability — data is only lost if both replicas fail simultaneously

**Memory capacity management**: Limit each Segment by size (e.g., 1GB) or time (e.g., 5 minutes). When the threshold is reached, the Segment is automatically sealed and a new one is created. Sealed in-memory Segments can be configured for async persistence to disk or S3, preserving data without impacting write latency.

### RocksDB (Local KV Persistent)

Local persistent storage backed by RocksDB, using a dedicated Column Family for messages, with write locks to avoid concurrent conflicts.

**Target scenarios**: Single-node deployment, local testing, edge scenarios. No cluster coordination required, zero external dependencies, reliable persistence.

**Limitation**: Data is not synchronized between nodes. In cluster mode, different Broker nodes cannot share data, making it unsuitable for production clusters.

### File Segment (Segmented File Log)

RobustMQ's core production-grade storage engine, designed for clustered, high-throughput, low-latency deployments. Kafka's typical scenarios (log collection, CDC, real-time data pipelines) are its primary use case.

---

## File Segment Core Design

File Segment is the most complex part of RobustMQ's storage layer. Here are its core design principles.

### Sequential Guarantee Under High Concurrency: I/O Pool

A fundamental requirement of message queue storage is that message Offsets within the same Partition must be strictly sequential. The traditional approach — one lock per Partition — holds the lock during disk I/O, causing severe contention at high concurrency, and each thread only writes one message before releasing, resulting in many small I/Os.

RobustMQ's solution is the **I/O Pool**: a fixed number of I/O Workers (e.g., 16) manage all Partitions. Through a fixed mapping of `partition_id % worker_count`, requests for the same Partition always go to the same Worker.

The Worker's core is batch processing: block waiting for the first request, then non-blockingly collect subsequent requests — potentially hundreds or thousands at once. These requests are grouped by partition and processed in batch, with a single fsync persisting hundreds or thousands of messages.

**Effect**: Transforms "one small write per message" into "one large write per batch"; replaces lock contention with queue ordering; memory usage depends only on the number of Workers, not the number of Partitions.

RobustMQ also uses Rust's `Bytes` type (Arc reference counting, clone doesn't copy data) for zero-copy — from network receive to disk write, the data itself exists in only one copy, with references held in different places. This reduces CPU and memory bandwidth pressure.

### Indexes: Sparse Index + Synchronous Construction

RobustMQ uses RocksDB to store four types of indexes: offset index, time index, key index, and tag index.

**Synchronous rather than asynchronous construction**: When a Worker batch-processes 1000 records, it simultaneously builds all indexes for those 1000 records and writes them via RocksDB's WriteBatch in a single operation. One I/O for data files, one I/O for indexes — two total. Under batch processing, index construction adds about 1ms, but delivers immediate index availability, strong data-index consistency, and simple crash recovery.

**Sparse offset index**: Rather than indexing every record, one index point is created per 1000 records, recording the file position for that Offset. On query, RocksDB locates the nearest index point, then sequential scan finds the target within at most 1000 records. 10 million records require only ~240KB of index space, with query latency around 2ms.

### Consistency Protocol: ISR over Quorum

With compute-storage separation, Segments are distributed across multiple Storage Nodes. Replica consistency is the core challenge. RobustMQ chose **ISR** (In-Sync Replicas) over Quorum.

**The problem with Quorum**: The Broker writes to 3 Storage Nodes in parallel and returns success when 2 confirm. But any replica may be incomplete — writing 1000 messages, replica A might be missing 100, B missing 120, C missing 80. During sequential consumption, a consumer may encounter a "gap" and mistakenly think consumption is complete, actually missing subsequent data. Fixing this requires complex background reconciliation logic, a high engineering risk that's hard to get right early on.

**The ISR choice**: Each Active Segment has one Leader that maintains an ISR list (set of in-sync replicas). A successful write means data has been replicated to all ISR replicas. Both the Leader and ISR Followers have complete data, reads succeed 100% of the time, no background repair needed. Followers pull data via Pull mode in batches, reducing network requests from millions to hundreds in high-QPS scenarios.

Through careful Segment distribution, Leaders are spread across different Storage Nodes, avoiding bottlenecks. The Broker layer remains stateless. Flexible acks configuration is also provided:

| acks Value | Semantics | Performance |
|------------|-----------|-------------|
| `all` | Wait for all ISR replicas | Strong consistency, slowest |
| `quorum` | Wait for majority | Balanced |
| `1` | Wait only for Leader | Fastest, may lose data |

### Active Segment vs Sealed Segment

With ISR, if every Segment has a Leader, the numbers become unmanageable: 1000 Shards × 100 Segments each = 100,000 Leaders to manage, with enormous election, ISR maintenance, and heartbeat overhead.

RobustMQ's solution is to distinguish two states:

**Active Segment**: The currently-written segment, with Leader and ISR mechanism, Followers continuously Pull and replicate.

**Sealed Segment**: When a Segment fills up (e.g., 1GB) or reaches a time threshold, the Leader waits for all ISR Followers to fully catch up and verifies consistency. Once all replicas are complete and consistent, it's marked Sealed and **the Leader role is released**. Sealed Segments have no Leader — all replicas are equal and reads can come from any replica.

**Effect**: Leader count = Shard count (not Segment count). 1000 Shards need only 1000 Leaders, reducing management overhead 100x. 99% of historical data reads spread across all Storage Nodes, fully utilizing storage resources.

### Scale Without Data Migration

Kafka scale-out requires copying Partition data between Brokers, taking hours or even days while impacting production performance.

The segmented design naturally solves this: **when adding a Storage Node, no historical data is migrated**. Just wait for the current Active Segment to fill up — new Segments are automatically assigned to the new node, immediately handling traffic. In high-traffic scenarios, a 1GB Segment fills in minutes to tens of minutes, so the new node joins quickly. The entire process is transparent to business operations with no performance loss.

Historical data stays on the original nodes as Sealed Segments, continuing to serve reads. When space needs to be freed, Sealed Segments can be asynchronously migrated to S3 in the background without impacting business.

### Tiered Storage

The immutable nature of Sealed Segments makes tiered storage implementation extremely simple: the file doesn't change, all replicas are complete — just read from any replica, upload to S3, update metadata. Retry on failure.

| Data Tier | Storage Location | Latency | Description |
|-----------|-----------------|---------|-------------|
| Hot (Active Segment) | Local SSD | Milliseconds | 3 replicas + ISR, lowest latency |
| Warm (Recent Sealed) | Local SSD/HDD | Milliseconds | Any replica readable |
| Cold (Historical Sealed) | S3/MinIO/HDFS | ~50ms | Extremely low cost, unlimited capacity |

For a workload of 1TB/day retained for 1 year: local storage costs ~$210,000; with tiered storage, this drops to ~$10,000 — a **95% reduction**.

Cold data migrated to S3 can also be converted to Parquet format, making it directly queryable by Spark, Hive, and other analytics tools. This naturally makes RobustMQ a data lake source, eliminating the need for ETL.

---

## Two Storage File Models

File Segment supports two underlying file organization strategies:

| Model | Description | Advantages | Disadvantages | Use Case |
|-------|-------------|-----------|---------------|---------|
| **Partition-per-file** | Each Partition has its own file (Kafka style) | Sequential reads/writes, peak throughput | Many small files at large Partition counts, metadata pressure | Low latency, high throughput, moderate Topic count |
| **Shared-file** | Multiple Partitions share a file (RocketMQ style) | Supports million-scale Partitions | Random read performance degradation | Massive Topics/Partitions, data isolation |

---

## Design Philosophy: Pragmatism

Looking back at the Storage Engine design, every technical decision was driven by a concrete problem:

- **ISR over Quorum**: Values the simplicity of data completeness guarantees; avoids complex background repair logic
- **Synchronous index construction**: Overhead is small under batch writes, but delivers immediate availability and strong consistency
- **Active/Sealed separation**: A simple optimization that dramatically reduces Leader management cost
- **Segmented design**: Naturally solves the data migration problem without additional mechanisms

We believe that for foundational components like a storage engine, **one should choose proven, mature solutions and make targeted optimizations on top, rather than pursuing theoretically perfect but engineering-risky approaches**. Combining existing techniques (sequential files, batch processing, ISR, sparse indexes) under a clear architecture with sound strategy choices — that's the right path.
