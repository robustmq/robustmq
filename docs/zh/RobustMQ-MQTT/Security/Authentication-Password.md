# Password 认证

Password 认证是当前最常用的方式，通过用户名和密码完成客户端身份校验。

## 配置说明

Password 认证的核心是 `authn_type = "password_based"`，并在 `storage_config` 中指定数据源。  
如果使用 MySQL，配置结构应与代码中的 `MysqlConfig` 对齐：

```rust
pub struct MysqlConfig {
    pub mysql_addr: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub query_user: String,
    pub query_acl: String,
    pub query_blacklist: String,
}
```

字段说明：

- `mysql_addr`：MySQL 地址（如 `127.0.0.1:3306`）
- `database`：数据库名
- `username` / `password`：连接账号与密码
- `query_user`：用户同步 SQL（认证核心数据来源）
- `query_acl`：ACL 同步 SQL
- `query_blacklist`：黑名单同步 SQL

也就是说，Password 认证并不强制你使用固定表结构，只要求查询结果能映射到系统约定字段。

当前支持的数据源包括：

- 内置数据源（Meta Service）
- MySQL
- PostgreSQL
- Redis
- HTTP

数据源的具体参数请参考：

- [数据源总览](./DataSource.md)
- [内置数据源（Meta Service）](./DataSource/BuiltIn.md)
- [MySQL 数据源](./DataSource/MySQL.md)
- [PostgreSQL 数据源](./DataSource/PostgreSQL.md)
- [Redis 数据源](./DataSource/Redis.md)
- [HTTP 数据源](./DataSource/HTTP.md)

## 使用说明

1. 在 Broker 配置中启用 `password_based` 认证；
2. 根据选定数据源完成配置；
3. 客户端连接时携带 `username/password`；
4. 认证通过后进入会话、订阅、发布等后续流程。

## 示例（概念示例）

```toml
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

- 生产环境建议关闭免密模式；
- 密码校验属于高频路径，建议保持缓存命中率；
- 数据源中的查询字段和类型需与系统约定一致。
