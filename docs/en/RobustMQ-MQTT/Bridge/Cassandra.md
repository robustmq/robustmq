# Cassandra Connector

## Overview

The Cassandra connector is a data integration component provided by RobustMQ for writing MQTT messages to Apache Cassandra distributed NoSQL databases. Cassandra is renowned for its high availability, scalability, and high write throughput, making it ideal for IoT time-series data storage, device registration management, and large-scale event logging.

This connector uses the [scylla](https://github.com/scylladb/scylla-rust-driver) Rust driver, which is compatible with both Apache Cassandra and ScyllaDB.

## Features

- Based on scylla Rust driver, async-native with Tokio
- Compatible with both Apache Cassandra and ScyllaDB
- Username/password authentication support
- Prepared Statements for efficient batch writes
- Multi-node cluster connection support
- Configurable connection timeout

## Table Schema

The connector writes data using CQL into a user-created table. Recommended table schema:

```sql
CREATE KEYSPACE IF NOT EXISTS mqtt
  WITH REPLICATION = {'class': 'SimpleStrategy', 'replication_factor': 1};

CREATE TABLE mqtt.mqtt_messages (
    msgid text,
    topic text,
    qos int,
    payload text,
    arrived bigint,
    PRIMARY KEY (msgid, topic)
);
```

### Field Description

| Field | Type | Description |
|-------|------|-------------|
| `msgid` | text | Message key (typically source identifier) |
| `topic` | text | Reserved field (empty string) |
| `qos` | int | Reserved field (default 0) |
| `payload` | text | MQTT message content |
| `arrived` | bigint | Message arrival timestamp |

## Configuration

### Connector Config

```rust
pub struct CassandraConnectorConfig {
    pub nodes: Vec<String>,       // Cassandra node addresses
    pub port: u16,                // Port number
    pub keyspace: String,         // Keyspace name
    pub table: String,            // Table name
    pub username: String,         // Username
    pub password: String,         // Password
    pub replication_factor: u32,  // Replication factor
    pub timeout_secs: u64,        // Connection timeout (seconds)
}
```

### Configuration Parameters

| Parameter | Type | Required | Default | Description | Example |
|-----------|------|----------|---------|-------------|---------|
| `nodes` | Array | Yes | - | List of Cassandra node addresses | `["127.0.0.1"]` |
| `port` | Number | No | `9042` | CQL native protocol port | `9042` |
| `keyspace` | String | Yes | - | Keyspace name | `mqtt` |
| `table` | String | Yes | - | Table name | `mqtt_messages` |
| `username` | String | No | empty | Authentication username | `cassandra` |
| `password` | String | No | empty | Authentication password | `cassandra` |
| `replication_factor` | Number | No | `1` | Replication factor | `3` |
| `timeout_secs` | Number | No | `15` | Connection timeout in seconds, range: 1-300 | `15` |

### Configuration Examples

#### Basic Configuration

```json
{
  "nodes": ["127.0.0.1"],
  "keyspace": "mqtt",
  "table": "mqtt_messages"
}
```

#### Authenticated Cluster Configuration

```json
{
  "nodes": ["cass-node1", "cass-node2", "cass-node3"],
  "port": 9042,
  "keyspace": "mqtt",
  "table": "mqtt_messages",
  "username": "admin",
  "password": "secret",
  "replication_factor": 3,
  "timeout_secs": 30
}
```

#### Full Connector Configuration

```json
{
  "cluster_name": "default",
  "connector_name": "cassandra_connector_01",
  "connector_type": "cassandra",
  "config": "{\"nodes\": [\"127.0.0.1\"], \"keyspace\": \"mqtt\", \"table\": \"mqtt_messages\"}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## Creating a Cassandra Connector with robust-ctl

### Basic Syntax

```bash
robust-ctl mqtt connector create \
  --connector-name <connector_name> \
  --connector-type <connector_type> \
  --config <config> \
  --topic-id <topic_id>
```

### Examples

#### 1. Single Node

```bash
robust-ctl mqtt connector create \
  --connector-name "cassandra_connector_01" \
  --connector-type "cassandra" \
  --config '{"nodes": ["127.0.0.1"], "keyspace": "mqtt", "table": "mqtt_messages"}' \
  --topic-id "sensor/data"
```

#### 2. Multi-Node Cluster

```bash
robust-ctl mqtt connector create \
  --connector-name "cassandra_cluster" \
  --connector-type "cassandra" \
  --config '{"nodes": ["cass1", "cass2", "cass3"], "port": 9042, "keyspace": "iot", "table": "device_events", "username": "admin", "password": "secret", "replication_factor": 3}' \
  --topic-id "device/#"
```

### Managing Connectors

```bash
# List all connectors
robust-ctl mqtt connector list

# View specific connector
robust-ctl mqtt connector list --connector-name "cassandra_connector_01"

# Delete connector
robust-ctl mqtt connector delete --connector-name "cassandra_connector_01"
```

### Full Example

#### Scenario: IoT Device Event Storage

```bash
# 1. Create Keyspace and Table in Cassandra
# cqlsh -e "CREATE KEYSPACE mqtt WITH REPLICATION = {'class': 'SimpleStrategy', 'replication_factor': 1}"
# cqlsh -e "CREATE TABLE mqtt.device_events (msgid text, topic text, qos int, payload text, arrived bigint, PRIMARY KEY (msgid, topic))"

# 2. Create the connector
robust-ctl mqtt connector create \
  --connector-name "device_to_cassandra" \
  --connector-type "cassandra" \
  --config '{"nodes": ["127.0.0.1"], "keyspace": "mqtt", "table": "device_events"}' \
  --topic-id "device/+"

# 3. View connectors
robust-ctl mqtt connector list
```

## Performance Optimization

### 1. Cluster Deployment
- Deploy at least 3 nodes in production
- Set appropriate `replication_factor` (typically 3)

### 2. Table Design & Partitioning
- Choose Partition Keys carefully to avoid hot partitions
- For time-series data, consider composite Partition Keys (e.g., `device_id` + date)

### 3. Batch Writes
- The connector has built-in batch writing, up to 100 records per batch
- Cassandra handles concurrent writes very efficiently

### 4. Security
- Enable authentication in production
- Create dedicated write-only users with limited permissions
- Consider enabling TLS for encrypted transport

## Monitoring and Troubleshooting

### 1. Check Connector Status

```bash
robust-ctl mqtt connector list --connector-name "cassandra_connector_01"
```

### 2. Common Issues

**Issue 1: Connection Failed**
- Verify Cassandra service is running
- Check node addresses and port (default 9042)
- Confirm network connectivity and firewall settings

**Issue 2: Keyspace/Table Not Found**
- Ensure Keyspace and Table are pre-created
- Check name spelling

**Issue 3: Authentication Failed**
- Verify username/password
- Confirm Cassandra has authentication enabled (`authenticator: PasswordAuthenticator`)

**Issue 4: Write Latency**
- Check Cassandra cluster load
- Add more nodes to distribute load
- Check network latency

## Summary

The Cassandra connector provides RobustMQ with integration capabilities for distributed NoSQL databases. Through the scylla driver, it achieves:

- **Dual Compatibility**: Supports both Apache Cassandra and ScyllaDB
- **High Availability**: Multi-node cluster connection with automatic failover
- **High Throughput**: Prepared Statements + batch writes, suitable for large-scale IoT data
- **Active Ecosystem**: scylla driver has an active community with 200k+ monthly downloads
