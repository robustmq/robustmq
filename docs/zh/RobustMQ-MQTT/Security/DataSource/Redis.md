# Redis 数据源

Redis 数据源适合低延迟读取场景，常用于用户和 ACL 数据已缓存到 Redis 的系统。

## 适用场景

- 已有统一 Redis 身份缓存，希望直接接入；
- 对读取延迟敏感，且数据结构相对简单；
- 希望作为外部数据源快速对接鉴权链路。

## 核心能力

- 基于约定的 key/hash 结构读取用户与 ACL；
- 通过缓存模式减少鉴权路径外部依赖开销；
- 便于和现有缓存体系对齐，不强制迁移到新存储。

## 运行方式（简要）

1. Broker 按 key 约定从 Redis 同步认证数据；
2. 同步结果更新本地缓存；
3. CONNECT 鉴权时以内存缓存判定为主。

## 配置说明

`redis_config` 关键字段：

- `redis_addr`：Redis 地址（如 `127.0.0.1:6379`）
- `mode`：`Single` / `Cluster` / `Sentinel`
- `database`：库编号
- `password`：连接密码
- `query`：查询模板（按你的存储模型定义）

## 使用说明

Redis 适配器会按约定的 key/hash 结构读取用户与 ACL。  
建议明确 key 命名规范，并保持字段定义稳定。

## 示例

```toml
[mqtt]

[[mqtt.auth]]
authn_type = "password_based"

[mqtt.auth.config.storage_config]
storage_type = "redis"

[mqtt.auth.config.storage_config.redis_config]
redis_addr = "127.0.0.1:6379"
mode = "Single"
database = 0
password = ""
query = "HMGET mqtt_user:${username} password_hash salt"
```

## 注意事项

- Redis 更适合在线读取与快速同步，不建议把复杂查询逻辑放在 Redis 层；
- 生产环境建议开启连接池与超时保护；
- 避免过大 key 或超大 hash，防止同步期间阻塞。
