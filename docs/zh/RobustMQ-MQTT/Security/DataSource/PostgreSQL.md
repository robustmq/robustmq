# PostgreSQL 数据源

PostgreSQL 数据源适用于认证和 ACL 已在 PG 维护的系统。

## 适用场景

- 业务主库是 PostgreSQL，希望直接复用已有身份数据；
- 对关系模型、事务能力、审计能力有要求；
- 希望在不改业务表结构前提下接入 Broker 鉴权。

## 核心能力

- 从 PostgreSQL 读取用户和 ACL 并同步到 Broker 缓存；
- 支持用查询语句适配已有数据模型；
- 将鉴权高频路径与数据库查询解耦。

## 运行方式（简要）

1. Broker 从 PostgreSQL 拉取认证相关数据；
2. 结果更新本地缓存；
3. 客户端连接时优先使用缓存判定鉴权结果。

## 配置说明

`postgres_config` 关键字段：

- `postgre_addr`：PostgreSQL 地址（如 `127.0.0.1:5432`）
- `database`：数据库名
- `username` / `password`：连接凭证
- `query`：查询语句（当前实现主要用于用户查询）

## 使用说明

PostgreSQL 适配器支持用户与 ACL 数据读取，并可用于同步到 Broker 缓存。  
建议先用最小查询验证数据映射是否正确，再上生产流量。

## 示例

```toml
[mqtt]

[[mqtt.auth]]
authn_type = "password_based"

[mqtt.auth.config.storage_config]
storage_type = "postgresql"

[mqtt.auth.config.storage_config.postgres_config]
postgre_addr = "127.0.0.1:5432"
database = "mqtt"
username = "postgres"
password = "postgres"
query = "SELECT password_hash, salt FROM mqtt_user where username = ${username} LIMIT 1"
```

## 注意事项

- 建议为用户名、客户端 ID、主题等字段建索引；
- 数据规模较大时，优先考虑批量/增量同步，而不是频繁全量扫描；
- 如果你有复杂表结构，建议通过视图统一输出，降低查询语句维护成本。
