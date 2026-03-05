# HTTP 数据源

HTTP 数据源适合已有统一认证中心、网关鉴权服务或外部 IAM 的场景。

## 适用场景

- 认证逻辑已经在外部服务中实现；
- 需要通过统一鉴权中心管理多系统接入；
- 希望 Broker 只负责调用鉴权 API，不重复实现业务规则。

## 核心能力

- 通过 HTTP 请求把认证判定委托给外部服务；
- 支持模板化参数替换（用户名、密码、客户端信息、来源 IP）；
- 可复用现有 IAM/网关体系，降低重复建设成本。

## 运行方式（简要）

1. 客户端 CONNECT 时，Broker 组装请求并调用认证 HTTP 接口；
2. 根据接口返回结果（allow/deny/ignore）决定连接处理；
3. 成功鉴权后进入后续会话与授权链路。

## 配置说明

`http_config` 关键字段：

- `url`：认证服务地址
- `method`：`GET` 或 `POST`
- `headers`：请求头模板（支持变量替换）
- `body`：请求体模板（支持变量替换）

常见变量包括：`${username}`、`${password}`、`${clientid}`、`${source_ip}`。

## 使用说明

连接时，Broker 按配置调用 HTTP 服务并解析返回结果：

- `allow`：认证通过
- `deny`：认证拒绝
- `ignore`：本次忽略（可由系统决定后续策略）

## 示例

```toml
[mqtt]

[[mqtt.auth]]
authn_type = "password_based"

[mqtt.auth.config.storage_config]
storage_type = "http"

[mqtt.auth.config.storage_config.http_config]
url = "http://127.0.0.1:8080/auth"
method = "POST"

[mqtt.auth.config.storage_config.http_config.headers]
content-type = "application/json"

[mqtt.auth.config.storage_config.http_config.body]
username = "${username}"
password = "${password}"
clientid = "${clientid}"
source_ip = "${source_ip}"
```

## 注意事项

- HTTP 服务可用性会影响认证能力，建议配置超时与熔断；
- 统一认证服务建议做水平扩容；
- 建议记录请求耗时与失败率，及时发现外部依赖抖动。
