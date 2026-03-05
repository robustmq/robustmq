# PostgreSQL 数据源

PostgreSQL 数据源适用于认证、ACL、黑名单已经在 PG 侧维护的系统。

## 适用场景

- 业务主库是 PostgreSQL，希望复用现有账号与权限数据；
- 需要保持关系模型和审计链路一致；
- 希望通过 SQL 适配已有表结构，而不是迁移到固定表。

## 核心能力

- 使用 `query_user/query_acl/query_blacklist` 三条查询分别同步三类数据；
- 按字段名映射读取结果（依赖别名），不依赖列顺序；
- 鉴权热路径走内存缓存，避免 CONNECT 期间频繁查库。

## 运行方式（简要）

1. Broker 周期性执行查询，从 PostgreSQL 拉取数据；
2. 结果写入本地缓存（用户、ACL、黑名单）；
3. 客户端连接时使用缓存判定，不在每次 CONNECT 时实时查询 PG。

## 配置说明

`postgres_config` 关键字段：

- `postgre_addr`：PostgreSQL 地址（如 `127.0.0.1:5432`）
- `database`：数据库名
- `username` / `password`：连接凭证
- `query_user`：用户同步 SQL
- `query_acl`：ACL 同步 SQL
- `query_blacklist`：黑名单同步 SQL

## 字段约定

### query_user 字段约定

结果中需包含：

- `username`
- `password`
- `salt`
- `is_superuser`（`1` 或 `0`）
- `created`（推荐 `YYYY-MM-DD HH:MM:SS` 字符串，或可转时间戳字符串）

### query_acl 字段约定

结果中需包含：

- `permission`（`1`=Allow，`0`=Deny）
- `ipaddr`
- `username`
- `clientid`
- `access`（`0..5`，对应 All/Subscribe/Publish/PubSub/Retain/Qos）
- `topic`

### query_blacklist 字段约定

结果中需包含：

- `blacklist_type`（`ClientId` / `User` / `Ip` / `ClientIdMatch` / `UserMatch` / `IPCIDR`）
- `resource_name`
- `end_time`（秒级时间戳，非负）
- `desc`

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
query_user = "SELECT username AS username, password AS password, salt AS salt, is_superuser AS is_superuser, created::text AS created FROM user_table"
query_acl = "SELECT permission AS permission, ipaddr AS ipaddr, username AS username, clientid AS clientid, access AS access, topic AS topic FROM acl_table"
query_blacklist = "SELECT blacklist_type AS blacklist_type, resource_name AS resource_name, end_time AS end_time, \"desc\" AS \"desc\" FROM blacklist_table"
```

## 注意事项

- 使用 `AS` 别名对齐字段名，列顺序无需固定；
- `created` 建议统一输出可解析格式，避免回退到当前时间；
- 建议为查询涉及字段建立索引，减少同步窗口内数据库压力。
