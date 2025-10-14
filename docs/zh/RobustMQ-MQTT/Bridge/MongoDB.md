# MongoDB 连接器

## 概述

MongoDB 连接器是 RobustMQ 提供的数据集成组件，用于将 MQTT 消息桥接到 MongoDB NoSQL 数据库系统。该连接器支持文档存储、灵活的数据模型和水平扩展，适合 IoT 数据存储、历史数据分析、非结构化数据存储和实时数据处理等场景。

## 配置说明

### 连接器配置

MongoDB 连接器使用 `MongoDBConnectorConfig` 结构进行配置：

```rust
pub struct MongoDBConnectorConfig {
    pub host: String,                        // MongoDB 服务器地址
    pub port: u16,                           // MongoDB 服务器端口
    pub database: String,                    // 数据库名称
    pub collection: String,                  // 集合名称
    pub username: Option<String>,            // 用户名（可选）
    pub password: Option<String>,            // 密码（可选）
    pub auth_source: Option<String>,         // 认证数据库（可选）
    pub deployment_mode: MongoDBDeploymentMode, // 部署模式
    pub replica_set_name: Option<String>,    // 副本集名称（可选）
    pub enable_tls: bool,                    // 启用 TLS/SSL
    pub max_pool_size: Option<u32>,          // 最大连接池大小
    pub min_pool_size: Option<u32>,          // 最小连接池大小
}
```

### 部署模式

MongoDB 连接器支持三种部署模式：

| 模式 | 说明 | 适用场景 |
|------|------|----------|
| `single` | 单机模式 | 开发测试环境 |
| `replicaset` | 副本集模式 | 生产环境（高可用） |
| `sharded` | 分片集群模式 | 大规模数据存储 |

### 配置参数

| 参数名 | 类型 | 必填 | 说明 | 示例 |
|--------|------|------|------|------|
| `host` | String | 是 | MongoDB 服务器地址 | `localhost` 或 `mongodb.example.com` |
| `port` | u16 | 否 | MongoDB 服务器端口，默认 27017 | `27017` |
| `database` | String | 是 | 数据库名称 | `mqtt_data` |
| `collection` | String | 是 | 集合名称 | `mqtt_messages` |
| `username` | String | 否 | 用户名 | `mqtt_user` |
| `password` | String | 否 | 密码 | `mqtt_pass` |
| `auth_source` | String | 否 | 认证数据库，默认 `admin` | `admin` |
| `deployment_mode` | String | 否 | 部署模式，默认 `single` | `single`/`replicaset`/`sharded` |
| `replica_set_name` | String | 否 | 副本集名称（副本集模式必需） | `rs0` |
| `enable_tls` | bool | 否 | 启用 TLS/SSL，默认 false | `true` 或 `false` |
| `max_pool_size` | u32 | 否 | 最大连接池大小 | `100` |
| `min_pool_size` | u32 | 否 | 最小连接池大小 | `10` |

### 配置示例

#### JSON 配置格式

**基本配置（单机模式）**：
```json
{
  "host": "localhost",
  "port": 27017,
  "database": "mqtt_data",
  "collection": "mqtt_messages",
  "username": "mqtt_user",
  "password": "mqtt_pass",
  "auth_source": "admin",
  "deployment_mode": "single",
  "enable_tls": false
}
```

**副本集配置**：
```json
{
  "host": "mongodb-0.mongodb.svc.cluster.local",
  "port": 27017,
  "database": "iot_data",
  "collection": "sensor_readings",
  "username": "iot_user",
  "password": "iot_pass",
  "auth_source": "admin",
  "deployment_mode": "replicaset",
  "replica_set_name": "rs0",
  "enable_tls": true,
  "max_pool_size": 100,
  "min_pool_size": 10
}
```

**无认证配置（开发环境）**：
```json
{
  "host": "localhost",
  "port": 27017,
  "database": "test_db",
  "collection": "test_messages",
  "deployment_mode": "single",
  "enable_tls": false
}
```

#### 完整连接器配置
```json
{
  "cluster_name": "default",
  "connector_name": "mongodb_connector_01",
  "connector_type": "MongoDB",
  "config": "{\"host\": \"localhost\", \"port\": 27017, \"database\": \"mqtt_data\", \"collection\": \"mqtt_messages\", \"username\": \"mqtt_user\", \"password\": \"mqtt_pass\", \"deployment_mode\": \"single\"}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## 数据模型

### 文档结构

MongoDB 连接器将 MQTT 消息转换为 BSON 文档存储，保留完整的消息结构：

```javascript
{
  "_id": ObjectId("507f1f77bcf86cd799439011"),  // MongoDB 自动生成
  "offset": 12345,                              // 消息偏移量
  "header": [                                   // 消息头
    {
      "name": "topic",
      "value": "sensor/temperature"
    },
    {
      "name": "qos",
      "value": "1"
    }
  ],
  "key": "sensor_001",                          // 消息键值（客户端 ID）
  "data": [123, 34, 116, 101, 109, 112, ...],   // 消息数据（字节数组）
  "tags": ["sensor", "temperature"],            // 消息标签
  "timestamp": 1640995200,                      // 消息时间戳（秒）
  "crc_num": 1234567890                         // CRC 校验值
}
```

### 字段说明

| 字段名 | 类型 | 说明 | 索引建议 |
|--------|------|------|----------|
| `_id` | ObjectId | MongoDB 文档唯一标识 | 自动索引 |
| `offset` | Number | 消息偏移量 | 建议索引 |
| `header` | Array | 消息头信息数组 | - |
| `key` | String | 消息键值（通常是客户端 ID） | 建议索引 |
| `data` | Binary | 消息数据（字节数组） | - |
| `tags` | Array | 消息标签数组 | 建议索引 |
| `timestamp` | Number | 消息时间戳（秒级） | 建议索引 |
| `crc_num` | Number | CRC 校验值 | - |

### 索引建议

为了提高查询性能，建议创建以下索引：

```javascript
// 1. 按时间戳查询索引
db.mqtt_messages.createIndex({ "timestamp": -1 })

// 2. 按客户端 ID 查询索引
db.mqtt_messages.createIndex({ "key": 1 })

// 3. 按标签查询索引
db.mqtt_messages.createIndex({ "tags": 1 })

// 4. 复合索引（客户端 + 时间）
db.mqtt_messages.createIndex({ "key": 1, "timestamp": -1 })

// 5. 按偏移量查询索引
db.mqtt_messages.createIndex({ "offset": 1 })

// 6. TTL 索引（自动删除过期数据）
db.mqtt_messages.createIndex(
  { "timestamp": 1 },
  { expireAfterSeconds: 2592000 }  // 30 天后自动删除
)
```

## 使用 robust-ctl 创建 MongoDB 连接器

### 基本语法

使用 `robust-ctl` 命令行工具可以方便地创建和管理 MongoDB 连接器：

```bash
robust-ctl mqtt connector create \
  --connector-name <连接器名称> \
  --connector-type <连接器类型> \
  --config <配置> \
  --topic-id <主题ID>
```

### 创建 MongoDB 连接器

#### 1. 基本创建命令

```bash
# 创建 MongoDB 连接器
robust-ctl mqtt connector create \
  --connector-name "mongodb_connector_01" \
  --connector-type "MongoDB" \
  --config '{"host": "localhost", "port": 27017, "database": "mqtt_data", "collection": "mqtt_messages", "username": "mqtt_user", "password": "mqtt_pass", "deployment_mode": "single"}' \
  --topic-id "sensor/data"
```

#### 2. 参数说明

| 参数 | 说明 | 示例值 |
|------|------|--------|
| `--connector-name` | 连接器名称，必须唯一 | `mongodb_connector_01` |
| `--connector-type` | 连接器类型，固定为 `MongoDB` | `MongoDB` |
| `--config` | JSON 格式的配置信息 | `{"host": "localhost", ...}` |
| `--topic-id` | 要监听的 MQTT 主题 | `sensor/data` |

#### 3. 高级配置示例

**副本集配置**：
```bash
robust-ctl mqtt connector create \
  --connector-name "mongodb_replicaset" \
  --connector-type "MongoDB" \
  --config '{"host": "mongodb-0.example.com", "port": 27017, "database": "iot_data", "collection": "sensor_data", "username": "iot_admin", "password": "secure_pass", "auth_source": "admin", "deployment_mode": "replicaset", "replica_set_name": "rs0", "enable_tls": true, "max_pool_size": 100}' \
  --topic-id "iot/sensors/+/data"
```

**高性能配置**：
```bash
robust-ctl mqtt connector create \
  --connector-name "mongodb_high_perf" \
  --connector-type "MongoDB" \
  --config '{"host": "mongodb.local", "port": 27017, "database": "high_throughput", "collection": "messages", "username": "perf_user", "password": "perf_pass", "max_pool_size": 200, "min_pool_size": 50}' \
  --topic-id "high/throughput/#"
```

### 管理连接器

#### 1. 列出所有连接器
```bash
# 列出所有连接器
robust-ctl mqtt connector list

# 列出指定名称的连接器
robust-ctl mqtt connector list --connector-name "mongodb_connector_01"
```

#### 2. 删除连接器
```bash
# 删除指定连接器
robust-ctl mqtt connector delete --connector-name "mongodb_connector_01"
```

### 完整操作示例

#### 场景：创建 IoT 数据存储系统

```bash
# 1. 创建传感器数据 MongoDB 连接器
robust-ctl mqtt connector create \
  --connector-name "iot_sensor_mongodb" \
  --connector-type "MongoDB" \
  --config '{"host": "mongodb.iot.local", "port": 27017, "database": "iot_platform", "collection": "sensor_readings", "username": "iot_writer", "password": "iot_pass_2023", "auth_source": "admin", "max_pool_size": 100}' \
  --topic-id "iot/sensors/+/readings"

# 2. 创建设备状态 MongoDB 连接器
robust-ctl mqtt connector create \
  --connector-name "device_status_mongodb" \
  --connector-type "MongoDB" \
  --config '{"host": "mongodb.iot.local", "port": 27017, "database": "iot_platform", "collection": "device_status", "username": "iot_writer", "password": "iot_pass_2023", "auth_source": "admin"}' \
  --topic-id "iot/devices/+/status"

# 3. 创建告警消息 MongoDB 连接器
robust-ctl mqtt connector create \
  --connector-name "alarm_mongodb" \
  --connector-type "MongoDB" \
  --config '{"host": "mongodb.iot.local", "port": 27017, "database": "iot_platform", "collection": "alarm_logs", "username": "iot_writer", "password": "iot_pass_2023", "auth_source": "admin"}' \
  --topic-id "iot/alarms/#"

# 4. 查看创建的连接器
robust-ctl mqtt connector list

# 5. 测试连接器（发布测试消息）
robust-ctl mqtt publish \
  --username "test_user" \
  --password "test_pass" \
  --topic "iot/sensors/temp_001/readings" \
  --qos 1 \
  --message '{"temperature": 25.5, "humidity": 60, "timestamp": 1640995200}'
```

## MongoDB 部署示例

### Docker 单机部署

```bash
# 启动 MongoDB 服务
docker run -d \
  --name mongodb \
  -p 27017:27017 \
  -e MONGO_INITDB_ROOT_USERNAME=admin \
  -e MONGO_INITDB_ROOT_PASSWORD=admin123 \
  -v mongodb_data:/data/db \
  mongo:7.0

# 等待服务启动
sleep 5

# 创建数据库和用户
docker exec -it mongodb mongosh -u admin -p admin123 --authenticationDatabase admin --eval "
  use mqtt_data;
  db.createUser({
    user: 'mqtt_user',
    pwd: 'mqtt_pass',
    roles: [{ role: 'readWrite', db: 'mqtt_data' }]
  });
"

# 创建集合和索引
docker exec -it mongodb mongosh -u mqtt_user -p mqtt_pass --authenticationDatabase mqtt_data --eval "
  use mqtt_data;
  db.createCollection('mqtt_messages');
  db.mqtt_messages.createIndex({ 'timestamp': -1 });
  db.mqtt_messages.createIndex({ 'key': 1 });
  db.mqtt_messages.createIndex({ 'tags': 1 });
"
```

### Docker Compose 部署

```yaml
version: '3.8'

services:
  mongodb:
    image: mongo:7.0
    container_name: mongodb
    ports:
      - "27017:27017"
    environment:
      MONGO_INITDB_ROOT_USERNAME: admin
      MONGO_INITDB_ROOT_PASSWORD: admin123
      MONGO_INITDB_DATABASE: mqtt_data
    volumes:
      - mongodb_data:/data/db
      - ./mongo-init.js:/docker-entrypoint-initdb.d/mongo-init.js:ro
    command: mongod --replSet rs0
    healthcheck:
      test: echo 'db.runCommand("ping").ok' | mongosh localhost:27017/test --quiet
      interval: 30s
      timeout: 10s
      retries: 5

  robustmq:
    image: robustmq/robustmq:latest
    container_name: robustmq
    ports:
      - "1883:1883"
      - "8883:8883"
    depends_on:
      - mongodb
    environment:
      - MONGODB_HOST=mongodb
      - MONGODB_PORT=27017

volumes:
  mongodb_data:
```

**mongo-init.js**：
```javascript
// 创建数据库和用户
db = db.getSiblingDB('mqtt_data');

db.createUser({
  user: 'mqtt_user',
  pwd: 'mqtt_pass',
  roles: [
    { role: 'readWrite', db: 'mqtt_data' }
  ]
});

// 创建集合
db.createCollection('mqtt_messages');

// 创建索引
db.mqtt_messages.createIndex({ 'timestamp': -1 });
db.mqtt_messages.createIndex({ 'key': 1 });
db.mqtt_messages.createIndex({ 'key': 1, 'timestamp': -1 });
db.mqtt_messages.createIndex({ 'tags': 1 });

// 创建 TTL 索引（30 天后自动删除）
db.mqtt_messages.createIndex(
  { 'timestamp': 1 },
  { expireAfterSeconds: 2592000 }
);
```

### MongoDB 副本集部署

```bash
# 启动副本集
docker-compose -f docker-compose-replica.yml up -d

# 初始化副本集
docker exec -it mongodb-0 mongosh --eval "
  rs.initiate({
    _id: 'rs0',
    members: [
      { _id: 0, host: 'mongodb-0:27017' },
      { _id: 1, host: 'mongodb-1:27017' },
      { _id: 2, host: 'mongodb-2:27017' }
    ]
  })
"
```

**docker-compose-replica.yml**：
```yaml
version: '3.8'

services:
  mongodb-0:
    image: mongo:7.0
    command: mongod --replSet rs0
    ports:
      - "27017:27017"
    volumes:
      - mongodb0_data:/data/db

  mongodb-1:
    image: mongo:7.0
    command: mongod --replSet rs0
    ports:
      - "27018:27017"
    volumes:
      - mongodb1_data:/data/db

  mongodb-2:
    image: mongo:7.0
    command: mongod --replSet rs0
    ports:
      - "27019:27017"
    volumes:
      - mongodb2_data:/data/db

volumes:
  mongodb0_data:
  mongodb1_data:
  mongodb2_data:
```

## 数据查询示例

### 基本查询

```javascript
// 查询最近 1 小时的消息
db.mqtt_messages.find({
  timestamp: { $gt: Math.floor(Date.now() / 1000) - 3600 }
}).sort({ timestamp: -1 })

// 查询特定客户端的消息
db.mqtt_messages.find({
  key: "sensor_001"
}).sort({ timestamp: -1 }).limit(100)

// 查询包含特定标签的消息
db.mqtt_messages.find({
  tags: "temperature"
})

// 查询时间范围内的消息
db.mqtt_messages.find({
  timestamp: {
    $gte: 1640995200,
    $lt: 1641081600
  }
})
```

### 聚合查询

```javascript
// 统计每小时的消息数量
db.mqtt_messages.aggregate([
  {
    $group: {
      _id: {
        $subtract: [
          "$timestamp",
          { $mod: ["$timestamp", 3600] }
        ]
      },
      count: { $sum: 1 }
    }
  },
  { $sort: { _id: -1 } }
])

// 统计各标签的消息数量
db.mqtt_messages.aggregate([
  { $unwind: "$tags" },
  {
    $group: {
      _id: "$tags",
      count: { $sum: 1 }
    }
  },
  { $sort: { count: -1 } }
])

// 统计各客户端的消息数量
db.mqtt_messages.aggregate([
  {
    $group: {
      _id: "$key",
      count: { $sum: 1 },
      first_message: { $min: "$timestamp" },
      last_message: { $max: "$timestamp" }
    }
  },
  { $sort: { count: -1 } }
])

// 按天统计消息量
db.mqtt_messages.aggregate([
  {
    $group: {
      _id: {
        $dateToString: {
          format: "%Y-%m-%d",
          date: { $toDate: { $multiply: ["$timestamp", 1000] } }
        }
      },
      count: { $sum: 1 }
    }
  },
  { $sort: { _id: -1 } }
])
```

### 高级查询

```javascript
// 查询消息头中包含特定字段的消息
db.mqtt_messages.find({
  "header.name": "topic",
  "header.value": { $regex: "^sensor/" }
})

// 使用文本搜索（需先创建文本索引）
db.mqtt_messages.createIndex({ "$**": "text" })
db.mqtt_messages.find({ $text: { $search: "temperature" } })

// 地理位置查询（如果数据包含坐标）
db.mqtt_messages.find({
  location: {
    $near: {
      $geometry: { type: "Point", coordinates: [120.0, 30.0] },
      $maxDistance: 1000
    }
  }
})
```

## 性能优化

### 数据库优化

#### 1. 索引优化

```javascript
// 查看当前索引
db.mqtt_messages.getIndexes()

// 分析查询性能
db.mqtt_messages.find({ key: "sensor_001" }).explain("executionStats")

// 删除未使用的索引
db.mqtt_messages.dropIndex("index_name")

// 创建覆盖索引
db.mqtt_messages.createIndex({ key: 1, timestamp: -1, tags: 1 })
```

#### 2. 分片配置

```javascript
// 启用分片
sh.enableSharding("mqtt_data")

// 按时间戳分片
sh.shardCollection(
  "mqtt_data.mqtt_messages",
  { timestamp: 1 }
)

// 按客户端 ID 分片（哈希分片）
sh.shardCollection(
  "mqtt_data.mqtt_messages",
  { key: "hashed" }
)
```

#### 3. 数据保留策略

```javascript
// 创建 TTL 索引（自动删除 30 天前的数据）
db.mqtt_messages.createIndex(
  { timestamp: 1 },
  { expireAfterSeconds: 2592000 }
)

// 手动归档旧数据
db.mqtt_messages.aggregate([
  { $match: { timestamp: { $lt: 1640995200 } } },
  { $out: "mqtt_messages_archive" }
])

// 删除归档后的数据
db.mqtt_messages.deleteMany({ timestamp: { $lt: 1640995200 } })
```

### 连接器优化

#### 1. 连接池配置

```json
{
  "max_pool_size": 100,
  "min_pool_size": 10
}
```

**配置建议**：
- 低并发场景：max_pool_size=20, min_pool_size=5
- 中等并发场景：max_pool_size=50, min_pool_size=10
- 高并发场景：max_pool_size=100+, min_pool_size=20

#### 2. 批量写入

MongoDB 连接器自动使用批量插入（`insert_many`），提高写入性能。

## 监控和故障排除

### 日志监控

连接器会输出详细的运行日志：

```
INFO  Successfully connected to MongoDB at localhost:27017
INFO  Successfully inserted 100 documents into MongoDB collection 'mqtt_messages'
ERROR Failed to connect to MongoDB at localhost:27017: connection timeout
ERROR Failed to insert documents into MongoDB collection 'mqtt_messages': write concern error
```

### MongoDB 监控命令

```javascript
// 查看数据库状态
db.stats()

// 查看集合状态
db.mqtt_messages.stats()

// 查看当前操作
db.currentOp()

// 查看慢查询
db.system.profile.find().sort({ ts: -1 }).limit(10)

// 启用慢查询分析
db.setProfilingLevel(1, { slowms: 100 })
```

### 常见问题

#### 1. 连接失败
```bash
# 检查 MongoDB 服务状态
docker exec mongodb mongosh --eval "db.adminCommand('ping')"

# 检查网络连接
telnet localhost 27017

# 查看 MongoDB 日志
docker logs mongodb
```

#### 2. 认证失败
```javascript
// 验证用户权限
db.auth("mqtt_user", "mqtt_pass")

// 查看用户信息
db.getUsers()

// 修改用户密码
db.changeUserPassword("mqtt_user", "new_password")
```

#### 3. 写入性能低
```javascript
// 查看写入延迟
db.serverStatus().opcounters

// 检查索引效率
db.mqtt_messages.find({ key: "sensor_001" }).explain("executionStats")

// 优化写入关注级别
db.mqtt_messages.insert(
  { ... },
  { writeConcern: { w: 1 } }
)
```

#### 4. 磁盘空间不足
```javascript
// 查看数据库大小
db.stats()

// 压缩集合
db.runCommand({ compact: "mqtt_messages" })

// 删除旧数据
db.mqtt_messages.deleteMany({
  timestamp: { $lt: Math.floor(Date.now() / 1000) - 2592000 }
})
```

## 总结

MongoDB 连接器是 RobustMQ 数据集成系统的重要组件，提供了灵活高效的文档存储能力。通过合理的配置和使用，可以满足 IoT 数据存储、历史数据分析、非结构化数据存储和实时数据处理等多种业务需求。

该连接器充分利用了 MongoDB 的文档模型、动态模式和水平扩展能力，结合 Rust 语言的内存安全和零成本抽象优势，实现了高性能、高可靠性的数据存储。支持单机、副本集和分片集群多种部署模式，是构建现代化 IoT 数据平台和大数据分析系统的重要工具。

### 关键特性

✅ **灵活的数据模型**：文档结构，无需预定义 Schema
✅ **水平扩展能力**：支持分片集群，轻松应对海量数据
✅ **高可用性**：副本集模式提供自动故障转移
✅ **丰富的查询能力**：支持复杂查询、聚合和地理位置查询
✅ **批量写入优化**：自动批量插入提高写入性能
✅ **数据生命周期管理**：TTL 索引自动清理过期数据
✅ **连接池管理**：智能连接池提升并发性能
