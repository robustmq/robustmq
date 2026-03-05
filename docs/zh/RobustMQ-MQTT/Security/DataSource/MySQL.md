# MySQL 数据源

MySQL 数据源适合已经把用户、ACL、黑名单维护在关系型数据库中的场景。

## 适用场景

- 账号体系已经落在 MySQL，需要直接复用现有表；
- 认证、授权、黑名单需要统一从数据库治理；
- 希望通过 SQL 适配旧系统，而不是迁移到固定表结构。

## 核心能力

- 通过 `query_user/query_acl/query_blacklist` 从 MySQL 同步数据；
- 支持“按查询结果映射”，不强制固定表名；
- 鉴权热路径走内存缓存，MySQL 主要承担同步来源。

## 运行方式（简要）

1. Broker 按配置执行查询，从 MySQL 拉取用户、ACL、黑名单数据；
2. 数据写入本地缓存；
3. CONNECT 鉴权时只比对内存缓存，不做每次实时查库。

## 配置说明

`mysql_config` 关键字段：

- `mysql_addr`：MySQL 地址（如 `127.0.0.1:3306`）
- `database`：数据库名
- `username` / `password`：连接凭证
- `query_user`：读取用户数据的 SQL
- `query_acl`：读取 ACL 数据的 SQL
- `query_blacklist`：读取黑名单数据的 SQL

## 使用说明

RobustMQ 会执行 `query_user/query_acl/query_blacklist` 并把结果映射到内部模型。  
因此你可以对接已有表，不必强制使用固定表名。

### query_user 返回列约定

需返回以下 5 列（顺序一致）：

1. `username` (String)
2. `password` (String)
3. `salt` (`Option<String>`)
4. `is_superuser` (0/1)
5. `created` (datetime 字符串)

### query_acl 返回列约定

需返回以下 6 列（顺序一致）：

1. `permission` (0/1)
2. `ipaddr` (String)
3. `username` (String)
4. `clientid` (String)
5. `access` (0..5)
6. `topic` (`Option<String>`)

### query_blacklist 返回列约定

需返回以下 4 列（顺序一致）：

1. `blacklist_type` (String)
2. `resource_name` (String)
3. `end_time` (u64)
4. `desc` (`Option<String>`)

## 示例

```toml
[mqtt]

[[mqtt.auth]]
authn_type = "password_based"

[mqtt.auth.config.storage_config]
storage_type = "mysql"

[mqtt.auth.config.storage_config.mysql_config]
mysql_addr = "127.0.0.1:3306"
database = "mqtt"
username = "root"
password = "123456"
query_user = "SELECT username,password,salt,is_superuser,created FROM user_table"
query_acl = "SELECT permission,ipaddr,username,clientid,access,topic FROM acl_table"
query_blacklist = "SELECT blacklist_type,resource_name,end_time,`desc` FROM blacklist_table"
```

## 注意事项

- 查询结果字段顺序与类型必须匹配；
- 建议给查询条件字段建立索引；
- 大规模场景建议用增量同步策略，避免频繁全量拉取。
