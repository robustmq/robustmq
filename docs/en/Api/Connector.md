# Connector Management HTTP API

> This document describes all HTTP API interfaces related to Connector management. Connectors are used to bridge MQTT messages to external systems.
>
> For general information, please refer to [COMMON.md](COMMON.md).

## API Endpoints

### 1. Connector List Query
- **Endpoint**: `GET /api/mqtt/connector/list`
- **Description**: Query connector list
- **Request Parameters**:
```json
{
  "connector_name": "kafka_connector",  // Optional, filter by connector name
  "limit": 20,
  "page": 1,
  "sort_field": "connector_name",
  "sort_by": "asc",
  "filter_field": "connector_type",
  "filter_values": ["kafka"],
  "exact_match": "false"
}
```

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "connector_name": "kafka_connector",
        "connector_type": "kafka",
        "config": "{\"bootstrap_servers\":\"localhost:9092\"}",
        "topic_name": "topic_001",
        "status": "Running",
        "broker_id": "1",
        "create_time": "2024-01-01 10:00:00",
        "update_time": "2024-01-01 11:00:00"
      }
    ],
    "total_count": 8
  }
}
```

**Field Descriptions**:
- `connector_name`: Connector name
- `connector_type`: Connector type
- `config`: Connector configuration (JSON string)
- `topic_name`: Associated MQTT topic
- `status`: Connector status (Idle, Running, Stopped)
- `broker_id`: Broker node ID running the connector
- `create_time`: Creation time
- `update_time`: Update time

---

### 2. Connector Detail Query
- **Endpoint**: `GET /api/mqtt/connector/detail`
- **Description**: Query detailed runtime status of a specific connector
- **Request Parameters**:
```json
{
  "connector_name": "kafka_connector"
}
```

- **Response Data Structure**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "last_send_time": 1698765432,
    "send_success_total": 10245,
    "send_fail_total": 3,
    "last_msg": "Batch sent successfully"
  }
}
```

- **Error Responses**:
```json
{
  "code": 1,
  "message": "Connector kafka_connector does not exist."
}
```

or

```json
{
  "code": 1,
  "message": "Connector thread kafka_connector does not exist."
}
```

**Field Descriptions**:
- `last_send_time`: Last send timestamp (Unix timestamp, seconds)
- `send_success_total`: Total successful messages sent
- `send_fail_total`: Total failed messages
- `last_msg`: Last operation message description, may be `null`

**Notes**:
- The connector must exist and be currently running to query details
- Statistics data will be reset when the connector restarts

---

### 3. Create Connector
- **Endpoint**: `POST /api/mqtt/connector/create`
- **Description**: Create a new connector
- **Request Parameters**:
```json
{
  "connector_name": "new_connector",
  "connector_type": "kafka",
  "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",
  "topic_name": "sensor/+",
  "failure_strategy": {
    "strategy": "discard",
    "retry_total_times": null,
    "wait_time_ms": null,
    "topic_name": null
  }
}
```

**Parameter Validation Rules**:
- `connector_name`: Length 1-128 characters
- `connector_type`: Length 1-50 characters, must be a supported type (see enum reference below)
- `config`: Length 1-4096 characters, JSON string
- `topic_name`: Length 1-256 characters, associated MQTT topic
- `failure_strategy`: Failure handling strategy (see below)

- **Response**: Returns `"success"` on success

---

### 4. Delete Connector
- **Endpoint**: `POST /api/mqtt/connector/delete`
- **Description**: Delete a connector
- **Request Parameters**:
```json
{
  "connector_name": "old_connector"
}
```

- **Response**: Returns `"success"` on success

---

## Connector Types and Configuration

### Supported Connector Types

| Type ID | Name | Category |
|---------|------|----------|
| `kafka` | Apache Kafka | Message Queue |
| `pulsar` | Apache Pulsar | Message Queue |
| `rabbitmq` | RabbitMQ | Message Queue |
| `mqtt` | MQTT Bridge | Message Queue |
| `redis` | Redis | Cache/Database |
| `mysql` | MySQL | Relational Database |
| `postgres` | PostgreSQL | Relational Database |
| `mongodb` | MongoDB | NoSQL Database |
| `clickhouse` | ClickHouse | Analytical Database |
| `cassandra` | Cassandra | Distributed Database |
| `elasticsearch` | Elasticsearch | Search Engine |
| `greptime` | GreptimeDB | Time-Series Database |
| `influxdb` | InfluxDB | Time-Series Database |
| `opentsdb` | OpenTSDB | Time-Series Database |
| `webhook` | Webhook (HTTP) | HTTP Push |
| `s3` | AWS S3 | Object Storage |
| `file` | Local File | File Storage |

---

### Kafka Connector

```json
{
  "connector_type": "kafka",
  "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}"
}
```

**Required Parameters**:
- `bootstrap_servers`: Kafka broker addresses, format `host1:port1,host2:port2`
- `topic`: Kafka topic name

**Optional Parameters**:
- `key`: Message key (default `""`), for partition routing, max 256 characters
- `compression_type`: Compression algorithm (default `"none"`), options: `none`, `gzip`, `snappy`, `lz4`, `zstd`
- `batch_size`: Max batch size in bytes (default `16384`), range 1-1048576
- `linger_ms`: Wait time before sending batch (default `5`), range 0-60000
- `acks`: Acknowledgment level (default `"1"`), options: `"0"`, `"1"`, `"all"`
- `retries`: Max retry attempts (default `3`), range 0-100
- `message_timeout_ms`: Message delivery timeout (default `30000`), range 1000-300000
- `cleanup_timeout_secs`: Shutdown flush timeout (default `10`), range 0-300

---

### Pulsar Connector

```json
{
  "connector_type": "pulsar",
  "config": "{\"server\":\"pulsar://localhost:6650\",\"topic\":\"mqtt-messages\"}"
}
```

**Required Parameters**:
- `server`: Pulsar broker address, format `pulsar://host:port` or `pulsar+ssl://host:port`
- `topic`: Pulsar topic name

**Authentication Parameters** (choose one):
- Token auth: `token`
- OAuth2 auth: `oauth` (JSON string)
- Basic auth: `basic_name` + `basic_password`

**Optional Parameters**:
- `connection_timeout_secs`: Connection timeout (default `30`), range 1-300
- `operation_timeout_secs`: Operation timeout (default `30`), range 1-300
- `send_timeout_secs`: Send timeout (default `30`), range 1-300
- `batch_size`: Batch record count (default `100`), range 1-10000
- `max_pending_messages`: Max pending messages (default `1000`), range 1-100000
- `compression`: Compression algorithm (default `"none"`), options: `none`, `lz4`, `zlib`, `zstd`, `snappy`

---

### RabbitMQ Connector

```json
{
  "connector_type": "rabbitmq",
  "config": "{\"server\":\"localhost\",\"username\":\"guest\",\"exchange\":\"mqtt_messages\"}"
}
```

**Required Parameters**:
- `server`: RabbitMQ server address
- `username`: Username
- `exchange`: Exchange name

**Optional Parameters**:
- `port`: Port (default `5672`)
- `password`: Password
- `virtual_host`: Virtual host (default `"/"`)
- `routing_key`: Routing key (default `""`)
- `delivery_mode`: Persistence mode (default `"NonPersistent"`), options: `NonPersistent`, `Persistent`
- `enable_tls`: Enable TLS (default `false`)
- `connection_timeout_secs`: Connection timeout (default `30`), range 1-300
- `heartbeat_secs`: Heartbeat interval (default `60`), range 0-300
- `channel_max`: Max channels (default `2047`)
- `frame_max`: Max frame size (default `131072`), range 4096-1048576
- `batch_size`: Batch record count (default `100`), range 1-10000
- `publisher_confirms`: Publisher confirms (default `true`)
- `confirm_timeout_secs`: Confirm timeout (default `30`), range 1-300

---

### MQTT Bridge Connector

```json
{
  "connector_type": "mqtt",
  "config": "{\"server\":\"mqtt://remote-broker:1883\"}"
}
```

**Required Parameters**:
- `server`: Remote MQTT Broker address, format `mqtt://host:port` or `mqtts://host:port`

**Optional Parameters**:
- `client_id_prefix`: Client ID prefix, max 64 characters
- `username`: Username
- `password`: Password
- `protocol_version`: Protocol version (default `"v5"`), options: `v3`, `v4`, `v5`
- `keepalive_secs`: Keep-alive interval (default `60`), range 1-65535
- `connect_timeout_secs`: Connection timeout (default `10`), range 1-300
- `enable_tls`: Enable TLS (default `false`)
- `topic_prefix`: Topic prefix (optional), prepended to original topic when forwarding
- `qos`: QoS level (default `1`), options: 0, 1, 2
- `retain`: Retain messages (default `false`)
- `max_retries`: Max retries (default `3`), max 10

---

### Redis Connector

```json
{
  "connector_type": "redis",
  "config": "{\"server\":\"127.0.0.1:6379\",\"command_template\":\"LPUSH mqtt_messages {payload}\"}"
}
```

**Required Parameters**:
- `server`: Redis server address, format `host:port`, comma-separated for cluster/sentinel
- `command_template`: Redis command template, max 4096 characters

**Optional Parameters**:
- `mode`: Operating mode (default `"single"`), options: `single`, `cluster`, `sentinel`
- `database`: Database number (default `0`), range 0-15
- `username`: Username
- `password`: Password
- `sentinel_master_name`: Sentinel master name (required for `sentinel` mode)
- `tls_enabled`: Enable TLS (default `false`)
- `connect_timeout_ms`: Connection timeout in ms (default `5000`)
- `pool_size`: Connection pool size (default `10`), range 1-100
- `max_retries`: Max retries (default `3`)
- `retry_interval_ms`: Retry interval in ms (default `1000`)

---

### MySQL Connector

```json
{
  "connector_type": "mysql",
  "config": "{\"host\":\"localhost\",\"port\":3306,\"database\":\"mqtt_data\",\"username\":\"root\",\"password\":\"password\",\"table\":\"mqtt_messages\"}"
}
```

**Required Parameters**:
- `host`: MySQL server address
- `port`: Port (default `3306`)
- `database`: Database name
- `username`: Username
- `password`: Password
- `table`: Table name

**Optional Parameters**:
- `pool_size`: Max pool connections (default `10`), range 1-1000
- `min_pool_size`: Min pool connections (default `2`)
- `connect_timeout_secs`: Connection timeout (default `10`), range 1-300
- `acquire_timeout_secs`: Acquire connection timeout (default `30`), range 1-300
- `idle_timeout_secs`: Idle timeout (default `600`), range 0-3600
- `max_lifetime_secs`: Max connection lifetime (default `1800`), range 0-7200
- `batch_size`: Batch record count (default `100`), range 1-10000
- `enable_batch_insert`: Batch insert mode (default `false`)
- `enable_upsert`: Upsert mode (default `false`), uses `ON DUPLICATE KEY UPDATE`
- `conflict_columns`: Conflict column names (required for upsert mode)
- `sql_template`: Custom SQL template with 3 `?` placeholders (`record_key`, `payload`, `timestamp`)

---

### PostgreSQL Connector

```json
{
  "connector_type": "postgres",
  "config": "{\"host\":\"localhost\",\"port\":5432,\"database\":\"mqtt_data\",\"username\":\"postgres\",\"password\":\"password\",\"table\":\"mqtt_messages\"}"
}
```

**Required Parameters**:
- `host`: PostgreSQL server address
- `port`: Port (default `5432`)
- `database`: Database name
- `username`: Username
- `password`: Password
- `table`: Table name

**Optional Parameters**:
- `pool_size`: Max pool connections (default `10`), range 1-1000
- `min_pool_size`: Min pool connections (default `2`)
- `connect_timeout_secs`: Connection timeout (default `10`), range 1-300
- `acquire_timeout_secs`: Acquire connection timeout (default `30`), range 1-300
- `idle_timeout_secs`: Idle timeout (default `600`), range 0-3600
- `max_lifetime_secs`: Max connection lifetime (default `1800`), range 0-7200
- `batch_size`: Batch record count (default `100`), range 1-10000
- `enable_batch_insert`: Batch insert mode (default `false`)
- `enable_upsert`: Upsert mode (default `false`), uses `ON CONFLICT ... DO UPDATE`
- `conflict_columns`: Conflict column names (required for upsert mode)
- `sql_template`: Custom SQL template with 5 `$1`-`$5` placeholders (`client_id`, `topic`, `timestamp`, `payload`, `data`)

---

### MongoDB Connector

```json
{
  "connector_type": "mongodb",
  "config": "{\"host\":\"localhost\",\"port\":27017,\"database\":\"mqtt_data\",\"collection\":\"mqtt_messages\"}"
}
```

**Required Parameters**:
- `host`: MongoDB server address
- `port`: Port (default `27017`)
- `database`: Database name
- `collection`: Collection name

**Optional Parameters**:
- `username`: Username
- `password`: Password
- `auth_source`: Authentication database (default `"admin"`)
- `deployment_mode`: Deployment mode (default `"single"`), options: `single`, `replicaset`, `sharded`
- `replica_set_name`: Replica set name (required for `replicaset` mode)
- `enable_tls`: Enable TLS (default `false`)
- `max_pool_size`: Max pool connections, range 1-1000
- `min_pool_size`: Min pool connections
- `connect_timeout_secs`: Connection timeout (default `10`), range 1-300
- `server_selection_timeout_secs`: Server selection timeout (default `30`), range 1-300
- `socket_timeout_secs`: Socket timeout (default `60`), range 1-600
- `batch_size`: Batch record count (default `100`), range 1-10000
- `ordered_insert`: Ordered insert (default `false`)
- `w`: Write concern (default `"1"`), options: `"0"`, `"1"`, `"majority"`

---

### ClickHouse Connector

```json
{
  "connector_type": "clickhouse",
  "config": "{\"url\":\"http://localhost:8123\",\"database\":\"mqtt\",\"table\":\"messages\"}"
}
```

**Required Parameters**:
- `url`: ClickHouse HTTP endpoint, must start with `http://` or `https://`
- `database`: Database name
- `table`: Table name

**Optional Parameters**:
- `username`: Username (default `""`)
- `password`: Password (default `""`)
- `pool_size`: Connection pool size (default `8`), range 1-64
- `timeout_secs`: Request timeout (default `15`), range 1-300

---

### Cassandra Connector

```json
{
  "connector_type": "cassandra",
  "config": "{\"nodes\":[\"127.0.0.1\"],\"keyspace\":\"mqtt\",\"table\":\"messages\"}"
}
```

**Required Parameters**:
- `nodes`: Cassandra node address list
- `keyspace`: Keyspace name
- `table`: Table name

**Optional Parameters**:
- `port`: Port (default `9042`)
- `username`: Username (default `""`)
- `password`: Password (default `""`)
- `replication_factor`: Replication factor (default `1`)
- `timeout_secs`: Timeout (default `15`), range 1-300

---

### Elasticsearch Connector

```json
{
  "connector_type": "elasticsearch",
  "config": "{\"url\":\"http://localhost:9200\",\"index\":\"mqtt_messages\"}"
}
```

**Required Parameters**:
- `url`: Elasticsearch server address
- `index`: Index name

**Optional Parameters**:
- `auth_type`: Authentication type (default `"none"`), options: `none`, `basic`, `apikey`
- `username`: Username (required for Basic auth)
- `password`: Password (required for Basic auth)
- `api_key`: API key (required for ApiKey auth)
- `enable_tls`: Enable TLS (default `false`)
- `ca_cert_path`: CA certificate path
- `timeout_secs`: Request timeout (default `30`), range 1-300
- `max_retries`: Max retries (default `3`), max 10

---

### GreptimeDB Connector

```json
{
  "connector_type": "greptime",
  "config": "{\"server_addr\":\"localhost:4000\",\"database\":\"public\"}"
}
```

**Required Parameters**:
- `server_addr`: GreptimeDB server address
- `database`: Database name

**Optional Parameters**:
- `user`: Username
- `password`: Password
- `precision`: Time precision (default `"Second"`)

---

### InfluxDB Connector

```json
{
  "connector_type": "influxdb",
  "config": "{\"server\":\"http://localhost:8086\",\"version\":\"v2\",\"token\":\"my-token\",\"org\":\"my-org\",\"bucket\":\"my-bucket\",\"measurement\":\"mqtt_data\"}"
}
```

**Required Parameters**:
- `server`: InfluxDB server address, must start with `http://` or `https://`
- `measurement`: Measurement name

**v2 Parameters** (when `version` is `"v2"`, default):
- `token`: API Token (required)
- `org`: Organization name (required)
- `bucket`: Bucket name (required)

**v1 Parameters** (when `version` is `"v1"`):
- `database`: Database name (required)
- `username`: Username
- `password`: Password

**Optional Parameters**:
- `version`: InfluxDB version (default `"v2"`), options: `v1`, `v2`
- `precision`: Write precision (default `"ms"`), options: `ns` (nanoseconds), `us` (microseconds), `ms` (milliseconds), `s` (seconds)
- `timeout_secs`: Request timeout (default `15`), range 1-300

---

### OpenTSDB Connector

```json
{
  "connector_type": "opentsdb",
  "config": "{\"server\":\"http://localhost:4242\"}"
}
```

**Required Parameters**:
- `server`: OpenTSDB server address, must start with `http://` or `https://`

**Optional Parameters**:
- `metric_field`: Metric field name in message (default `"metric"`)
- `value_field`: Value field name in message (default `"value"`)
- `tags_fields`: Tag field name list in message (default `[]`)
- `timeout_secs`: Request timeout (default `30`), range 1-300
- `max_retries`: Max retries (default `3`), max 10
- `summary`: Return summary info (default `false`)
- `details`: Return detailed info (default `false`)

---

### Webhook Connector

```json
{
  "connector_type": "webhook",
  "config": "{\"url\":\"https://example.com/webhook\"}"
}
```

**Required Parameters**:
- `url`: Webhook target URL, must start with `http://` or `https://`, max 2048 characters

**Optional Parameters**:
- `method`: HTTP method (default `"post"`), options: `post`, `put`
- `headers`: Custom request headers (default `{}`)
- `timeout_ms`: Request timeout in ms (default `5000`), range 1-60000
- `auth_type`: Authentication type (default `"none"`), options: `none`, `basic`, `bearer`
- `username`: Username (required for Basic auth)
- `password`: Password (required for Basic auth)
- `bearer_token`: Bearer Token (required for Bearer auth)

---

### AWS S3 Connector

```json
{
  "connector_type": "s3",
  "config": "{\"bucket\":\"my-mqtt-bucket\",\"region\":\"us-east-1\",\"object_key_prefix\":\"mqtt\"}"
}
```

**Required Parameters**:
- `bucket`: S3 bucket name
- `region`: AWS region (for example `us-east-1`)

**Optional Parameters**:
- `endpoint`: S3 endpoint (for MinIO and other S3-compatible storage)
- `access_key_id`: Access key (must be used together with `secret_access_key`)
- `secret_access_key`: Secret key (must be used together with `access_key_id`)
- `session_token`: Session token for temporary credentials
- `root`: Root path prefix in object storage (default `""`)
- `object_key_prefix`: Object key prefix (default `"mqtt"`)
- `file_extension`: Object suffix (default `"json"`, alphanumeric only)

> Write behavior: The S3 connector serializes one batch into a JSON array and writes it as one object.

---

### Local File Connector

```json
{
  "connector_type": "file",
  "config": "{\"local_file_path\":\"/tmp/mqtt_messages.log\"}"
}
```

**Required Parameters**:
- `local_file_path`: File path

**Optional Parameters**:
- `rotation_strategy`: File rotation strategy (default `"none"`), options: `none`, `size`, `hourly`, `daily`
- `max_size_gb`: Max file size in GB (default `1`, range 1-10, only for `size` strategy)

---

## Failure Handling Strategy

The `failure_strategy` parameter defines how the connector handles message delivery failures.

### Strategy Types

#### 1. Discard Strategy

Immediately discards failed messages without retry.

```json
{
  "strategy": "discard"
}
```

#### 2. Discard After Retry Strategy

Retries delivery for a specified number of times, then discards.

```json
{
  "strategy": "discard_after_retry",
  "retry_total_times": 3,
  "wait_time_ms": 1000
}
```

- `retry_total_times`: Maximum retry attempts (excluding the initial send, must be > 0)
- `wait_time_ms`: Wait time in milliseconds between retries (must be > 0)

#### 3. Dead Message Queue Strategy

Retries first, and after retries are exhausted, writes failed messages to a designated dead letter queue topic for later recovery and analysis.

```json
{
  "strategy": "dead_message_queue",
  "topic_name": "dead_letter_queue",
  "retry_total_times": 3,
  "wait_time_ms": 1000
}
```

- `topic_name`: Dead letter queue topic name (default `"dead_letter_queue"`)
- `retry_total_times`: Maximum retry attempts before writing to dead letter queue (excluding the initial send, default `3`, must be > 0)
- `wait_time_ms`: Wait time in milliseconds between retries (default `1000`, must be > 0)

> Validation: `topic_name` for `dead_message_queue` (when provided) cannot be empty and must be <= 256 characters.

**Dead Letter Message Format**:

Messages written to the dead letter queue are in JSON format as `DeadLetterRecord`:

```json
{
  "connector_name": "kafka_bridge",
  "source_topic": "sensor/+",
  "error_message": "Connection refused",
  "retry_times": 3,
  "original_key": "sensor-001",
  "original_data": [98, 121, 116, 101, 115],
  "original_timestamp": 1640995200,
  "dead_letter_timestamp": 1640995260
}
```

| Field | Type | Description |
|-------|------|-------------|
| `connector_name` | String | Name of the failed connector |
| `source_topic` | String | Original message source topic |
| `error_message` | String | Failure reason |
| `retry_times` | u32 | Number of retries performed |
| `original_key` | String? | Original message key (may be null) |
| `original_data` | [u8] | Original message data (byte array) |
| `original_timestamp` | u64 | Original message timestamp |
| `dead_letter_timestamp` | u64 | Timestamp when entered dead letter queue |

---

## Enum Reference

### Connector Type (connector_type)

| Value | Name | Category |
|-------|------|----------|
| `kafka` | Apache Kafka | Message Queue |
| `pulsar` | Apache Pulsar | Message Queue |
| `rabbitmq` | RabbitMQ | Message Queue |
| `mqtt` | MQTT Bridge | Message Bridge |
| `redis` | Redis | Cache/Database |
| `mysql` | MySQL | Relational Database |
| `postgres` | PostgreSQL | Relational Database |
| `mongodb` | MongoDB | NoSQL Database |
| `clickhouse` | ClickHouse | Analytical Database |
| `cassandra` | Cassandra | Distributed Database |
| `elasticsearch` | Elasticsearch | Search Engine |
| `greptime` | GreptimeDB | Time-Series Database |
| `influxdb` | InfluxDB | Time-Series Database |
| `opentsdb` | OpenTSDB | Time-Series Database |
| `webhook` | Webhook (HTTP) | HTTP Push |
| `s3` | AWS S3 | Object Storage |
| `file` | Local File | File Storage |

### Connector Status (status)

| Value | Description |
|-------|-------------|
| `Idle` | Idle, not assigned to a Broker |
| `Running` | Running |
| `Stopped` | Stopped |

### Failure Handling Strategy (strategy)

| Value | Description |
|-------|-------------|
| `discard` | Discard immediately |
| `discard_after_retry` | Discard after retry |
| `dead_message_queue` | Write to dead letter queue |

### Redis Mode (mode)

| Value | Description |
|-------|-------------|
| `single` | Single node mode |
| `cluster` | Cluster mode |
| `sentinel` | Sentinel mode |

### Webhook HTTP Method (method)

| Value | Description |
|-------|-------------|
| `post` | HTTP POST |
| `put` | HTTP PUT |

### Webhook Auth Type (auth_type)

| Value | Description |
|-------|-------------|
| `none` | No authentication |
| `basic` | Basic authentication |
| `bearer` | Bearer Token authentication |

### MQTT Bridge Protocol Version (protocol_version)

| Value | Description |
|-------|-------------|
| `v3` | MQTT 3.1 |
| `v4` | MQTT 3.1.1 |
| `v5` | MQTT 5.0 |

### InfluxDB Version (version)

| Value | Description |
|-------|-------------|
| `v1` | InfluxDB 1.x |
| `v2` | InfluxDB 2.x |

### InfluxDB Write Precision (precision)

| Value | Description |
|-------|-------------|
| `ns` | Nanoseconds |
| `us` | Microseconds |
| `ms` | Milliseconds |
| `s` | Seconds |

### MongoDB Deployment Mode (deployment_mode)

| Value | Description |
|-------|-------------|
| `single` | Single node |
| `replicaset` | Replica set |
| `sharded` | Sharded cluster |

---

## Usage Examples

### Query Connector List
```bash
curl "http://localhost:8080/api/mqtt/connector/list?limit=10&page=1"
```

### Query Connector Detail
```bash
curl "http://localhost:8080/api/mqtt/connector/detail?connector_name=kafka_bridge"
```

### Create Kafka Connector
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "connector_name": "kafka_bridge",
    "connector_type": "kafka",
    "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",
    "topic_name": "sensor/+",
    "failure_strategy": {
      "strategy": "discard"
    }
  }'
```

### Create Connector with Retry Strategy
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "connector_name": "kafka_bridge_retry",
    "connector_type": "kafka",
    "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",
    "topic_name": "sensor/+",
    "failure_strategy": {
      "strategy": "discard_after_retry",
      "retry_total_times": 5,
      "wait_time_ms": 2000
    }
  }'
```

### Create Connector with Dead Letter Queue
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "connector_name": "kafka_bridge_dlq",
    "connector_type": "kafka",
    "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",
    "topic_name": "sensor/+",
    "failure_strategy": {
      "strategy": "dead_message_queue",
      "topic_name": "dead_letter_queue",
      "retry_total_times": 5,
      "wait_time_ms": 2000
    }
  }'
```

### Create Redis Connector
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "connector_name": "redis_bridge",
    "connector_type": "redis",
    "config": "{\"server\":\"127.0.0.1:6379\",\"command_template\":\"LPUSH mqtt_messages {payload}\"}",
    "topic_name": "sensor/+",
    "failure_strategy": {
      "strategy": "discard"
    }
  }'
```

### Create Webhook Connector
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "connector_name": "webhook_bridge",
    "connector_type": "webhook",
    "config": "{\"url\":\"https://example.com/webhook\",\"method\":\"post\",\"auth_type\":\"bearer\",\"bearer_token\":\"my-token\"}",
    "topic_name": "sensor/+",
    "failure_strategy": {
      "strategy": "discard"
    }
  }'
```

### Create InfluxDB Connector (v2)
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "connector_name": "influxdb_bridge",
    "connector_type": "influxdb",
    "config": "{\"server\":\"http://localhost:8086\",\"version\":\"v2\",\"token\":\"my-token\",\"org\":\"my-org\",\"bucket\":\"mqtt\",\"measurement\":\"sensor_data\"}",
    "topic_name": "sensor/+",
    "failure_strategy": {
      "strategy": "discard"
    }
  }'
```

### Create AWS S3 Connector
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "connector_name": "s3_bridge",
    "connector_type": "s3",
    "config": "{\"bucket\":\"my-mqtt-bucket\",\"region\":\"us-east-1\",\"object_key_prefix\":\"mqtt\"}",
    "topic_name": "sensor/+",
    "failure_strategy": {
      "strategy": "discard"
    }
  }'
```

### Delete Connector
```bash
curl -X POST http://localhost:8080/api/mqtt/connector/delete \
  -H "Content-Type: application/json" \
  -d '{
    "connector_name": "old_connector"
  }'
```

---

*Documentation Version: v1.0*  
*Last Updated: 2026-03-03*  
*Based on Code Version: RobustMQ v0.3.2*
