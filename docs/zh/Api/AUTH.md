# Admin HTTP API 鉴权说明

> 本文档介绍 RobustMQ Admin HTTP API 的鉴权机制、登录接口及配置方法。

## 概述

RobustMQ Admin Server 采用基于 JWT（JSON Web Token）的鉴权方案：

- **本地访问免密**：来自 `127.0.0.1` 或 `::1` 的请求直接放行，无需任何 token（适用于本地运维、CLI 工具、本机 curl）
- **远程访问必须鉴权**：来自其他 IP 的请求必须在 HTTP 请求头中携带有效的 Bearer token
- **例外路径**：`/api/v1/login`、`/health/*`、`/metrics` 无需鉴权，任何来源均可访问

---

## 鉴权流程

```text
客户端
  │
  ├─ 本地 (127.0.0.1 / ::1)
  │    └─ 直接访问所有接口，无需 token ✅
  │
  └─ 远程 IP
       │
       ├─ POST /api/v1/login  →  获取 JWT token
       │
       └─ 携带 Authorization: Bearer <token> 访问其他接口
```

---

## 登录接口

### `POST /api/v1/login`

使用用户名和密码换取 JWT token。

**请求体：**

```json
{
  "username": "admin",
  "password": "admin"
}
```

**成功响应（HTTP 200）：**

```json
{
  "code": 0,
  "message": "success",
  "data": {
    "token": "<JWT_TOKEN>",
    "expires_in": 28800
  }
}
```

| 字段 | 类型 | 说明 |
|---|---|---|
| `token` | string | JWT token，用于后续请求鉴权 |
| `expires_in` | number | token 有效期（秒），默认 8 小时 = 28800 秒 |

**失败响应（HTTP 401）：**

```json
{
  "code": 401,
  "message": "Invalid username or password",
  "data": null
}
```

---

## 携带 Token 访问接口

获取 token 后，在所有 API 请求的 HTTP 请求头中添加：

```http
Authorization: Bearer <token>
```

**curl 示例：**

```bash
# 1. 登录获取 token（远程访问时）
TOKEN=$(curl -s -X POST http://<host>:8080/api/v1/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"<your-password>"}' \
  | jq -r '.data.token')

# 2. 携带 token 访问接口
curl -H "Authorization: Bearer $TOKEN" \
  http://<host>:8080/api/mqtt/overview
```

**本地访问（免 token）：**

```bash
# 本地直接访问，无需任何 token
curl http://127.0.0.1:8080/api/mqtt/overview
```

---

## 配置说明

鉴权相关配置在 `config/server.toml` 的 `[admin]` 节：

```toml
[admin]
# 管理员用户名，默认 admin
username = "admin"

# 管理员密码，生产环境务必修改为强密码
password = "<your-password>"

# JWT 签名密钥（HMAC-SHA256）
# 生产环境务必修改为随机字符串，长度建议 32 位以上
jwt_secret = "<your-jwt-secret>"

# Token 有效期（小时），默认 8 小时
token_ttl_hours = 8
```

> ⚠️ **安全提示**：生产环境部署时，请务必修改 `password` 和 `jwt_secret`，避免使用默认值。

---

## 各客户端集成说明

### Dashboard 前端

Dashboard 内置登录页面，输入用户名/密码后自动调用 `/api/v1/login` 获取 token，并在后续请求中自动携带。Token 过期后自动跳转回登录页。

### curl / 脚本

```bash
# 本地机器（服务器上直接运行）：无需 token
curl http://127.0.0.1:8080/api/cluster/config/get

# 远程机器：先登录获取 token
TOKEN=$(curl -s -X POST http://<host>:8080/api/v1/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"<your-password>"}' \
  | jq -r '.data.token')

curl -H "Authorization: Bearer $TOKEN" \
  http://<host>:8080/api/cluster/config/get
```

### CLI 工具（cli-command）

CLI 工具默认连接 `127.0.0.1`，属于本地访问，无需鉴权配置，直接使用即可。

如需连接远程节点，使用 `--token` 参数传入 token（或在配置文件中指定）。

---

## Token 错误说明

| HTTP 状态码 | code 字段 | 含义 |
|---|---|---|
| 401 | 401 | 未提供 token、token 无效或已过期 |
| 200 | 0 | 鉴权通过，正常响应 |

token 过期后需重新调用 `/api/v1/login` 获取新 token。
