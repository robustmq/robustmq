# MongoDB 数据源

MongoDB 数据源适用于认证、ACL、黑名单已经在 MongoDB 中维护的场景。

## 适用场景

- 已有用户中心或设备平台使用 MongoDB 存储身份策略；
- 希望通过文档过滤条件对接现有集合，而不改表结构；
- 需要保持 Broker 的缓存优先鉴权模型。

## 核心能力

- 从 MongoDB 拉取用户、ACL、黑名单三类数据；
- 支持分别配置 `collection_user/collection_acl/collection_blacklist`；
- 支持分别配置 `query_user/query_acl/query_blacklist`（JSON 过滤条件）。

## 运行方式（简要）

1. Broker 周期性读取 MongoDB；
2. 将结果写入本地缓存；
3. 连接与访问控制判定以缓存为主，不在热路径实时查 MongoDB。

## 配置说明

`mongodb_config` 关键字段：

- `mongodb_uri`：MongoDB 连接地址
- `database`：数据库名
- `username` / `password`：认证信息（可选，不填则按 URI）
- `collection_user`：用户集合名
- `collection_acl`：ACL 集合名
- `collection_blacklist`：黑名单集合名
- `query_user`：用户过滤条件（JSON 字符串）
- `query_acl`：ACL 过滤条件（JSON 字符串）
- `query_blacklist`：黑名单过滤条件（JSON 字符串）

## 字段约定

### 用户文档

至少需要：

- `username`
- `password`

可选：

- `salt`
- `is_superuser`（`bool` / `0|1` / `"true"|"false"`）
- `created`（DateTime / 时间戳 / 可解析字符串）

### ACL 文档

至少需要：

- `permission`（`1|0` 或 `Allow|Deny`）
- `action` 或 `access`（`0..5` 或 `All|Subscribe|Publish|PubSub|Retain|Qos`）
- `topic` 或 `topics`（数组）

可选：

- `username`
- `clientid`
- `ipaddr`（或 `ipaddress`）

### 黑名单文档

至少需要：

- `blacklist_type`（`ClientId` / `User` / `Ip` / `ClientIdMatch` / `UserMatch` / `IPCIDR`）
- `resource_name`
- `end_time`

可选：

- `desc`

## 示例

```toml
[[mqtt.auth]]
authn_type = "password_based"

[mqtt.auth.config.storage_config]
storage_type = "mongodb"

[mqtt.auth.config.storage_config.mongodb_config]
mongodb_uri = "mongodb://127.0.0.1:27017"
database = "mqtt"
username = ""
password = ""
collection_user = "mqtt_user"
collection_acl = "mqtt_acl"
collection_blacklist = "mqtt_blacklist"
query_user = "{}"
query_acl = "{}"
query_blacklist = "{}"
```

## 注意事项

- `query_*` 必须是合法 JSON（例如 `{}`、`{"enabled": true}`）；
- 建议为 `username`、`clientid`、`resource_name` 等检索字段建索引；
- 统一 `blacklist_type` 枚举值，避免大小写和命名不一致导致数据被跳过。
