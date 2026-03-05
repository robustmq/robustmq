# HTTP 数据源

HTTP 数据源适合已有统一账号平台或策略服务，并希望通过 HTTP 拉取用户、ACL、黑名单数据的场景。

## 适用场景

- 用户、权限、黑名单数据由外部平台维护；
- 希望通过 HTTP 接口接入，而不是直接连数据库；
- 需要保留 Broker 的缓存优先鉴权模型。

## 核心能力

- 通过 HTTP 拉取三类数据：`user` / `acl` / `blacklist`；
- 支持为三类数据分别配置 endpoint（`query_user/query_acl/query_blacklist`）；
- 支持响应为数组，或对象包装数组（如 `users`、`acls`、`blacklists`、`data`）。

## 运行方式（简要）

1. Broker 周期性调用 HTTP endpoint 拉取数据；
2. 解析后更新本地缓存；
3. CONNECT/Publish/Subscribe 判定时优先使用缓存，不在热路径发起 HTTP 请求。

## 配置说明

`http_config` 关键字段：

- `url`：基础服务地址
- `method`：`GET` 或 `POST`
- `query_user`：用户数据 endpoint（可填完整 URL 或相对路径）
- `query_acl`：ACL 数据 endpoint（可填完整 URL 或相对路径）
- `query_blacklist`：黑名单数据 endpoint（可填完整 URL 或相对路径）
- `headers`：请求头模板
- `body`：请求体模板
模板变量支持 `${resource}`，对应 `user` / `acl` / `blacklist`。

## 使用说明

响应支持以下结构：

- 直接返回数组：`[{...}, {...}]`
- 对象包装数组：`{ "users": [...] }`、`{ "acls": [...] }`、`{ "blacklists": [...] }`
- 通用包装：`{ "data": [...] }`

## 示例

```toml
[mqtt]

[[mqtt.auth]]
authn_type = "password_based"

[mqtt.auth.config.storage_config]
storage_type = "http"

[mqtt.auth.config.storage_config.http_config]
url = "http://127.0.0.1:8080/auth-source"
method = "POST"
query_user = "/users"
query_acl = "/acls"
query_blacklist = "/blacklists"

[mqtt.auth.config.storage_config.http_config.headers]
content-type = "application/json"

[mqtt.auth.config.storage_config.http_config.body]
resource = "${resource}"
```

## 注意事项

- HTTP 数据源承担“同步”职责，不是每个连接实时鉴权；
- 建议将 `query_user/query_acl/query_blacklist` 拆分成独立 endpoint，便于隔离问题；
- 建议统一响应字段命名，避免不同资源返回格式不一致导致解析失败。
