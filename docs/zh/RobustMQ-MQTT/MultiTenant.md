# MQTT 多租户

## 什么是多租户？

多租户是 RobustMQ MQTT 提供的资源隔离机制，允许多个独立的业务或团队共享同一套 MQTT 服务，同时保持各自的数据、订阅、会话和权限相互隔离。

每个租户拥有独立的：

- 客户端连接和会话
- 主题（Topic）命名空间
- 订阅管理
- 访问控制（ACL）和黑名单
- 自动订阅规则

## 如何在连接时指定租户

客户端在发起 MQTT CONNECT 时，通过以下三种方式之一传递租户名称。系统按优先级从高到低依次识别：

### 方式一：MQTT v5 User Property（最高优先级）

在 CONNECT 包的 User Properties 中添加键值对 `tenant = <租户名>`。

```bash
# 使用 MQTTX CLI 连接到 acme 租户
mqttx conn \
  --mqtt-version 5 \
  -h 127.0.0.1 -p 1883 \
  -i device-001 \
  --user-properties "tenant: acme"
```

适用场景：客户端使用 MQTT v5 协议，且希望 client_id 和 username 不携带租户前缀。

---

### 方式二：Username 前缀（次高优先级）

将租户名作为 username 的前缀，用 `@` 分隔：格式为 `<租户名>@<真实用户名>`。

```bash
# username 为 acme@alice，租户 = acme，真实用户名 = alice
mqttx conn \
  -h 127.0.0.1 -p 1883 \
  -i device-001 \
  -u "acme@alice" \
  -P "password"
```

适用场景：客户端使用 MQTT v3.1/v3.1.1，通过 username 携带租户信息。

---

### 方式三：Client ID 前缀（第三优先级）

将租户名作为 client_id 的前缀，用 `@` 分隔：格式为 `<租户名>@<真实 Client ID>`。

```bash
# client_id 为 acme@device-001，租户 = acme，真实 client_id = device-001
mqttx conn \
  -h 127.0.0.1 -p 1883 \
  -i "acme@device-001"
```

适用场景：客户端无法修改 username，但可以控制 client_id 的格式。

---

### 方式四：默认租户（兜底）

如果以上三种方式均未携带租户信息，连接将归属到系统默认租户（`default`）。

```bash
# 不携带任何租户信息，使用默认租户
mqttx conn \
  -h 127.0.0.1 -p 1883 \
  -i device-001
```

---

## 优先级总结

| 优先级 | 来源 | 示例 |
|--------|------|------|
| 1（最高）| MQTT v5 User Property | `tenant: acme` |
| 2 | Username 前缀 | `acme@alice` |
| 3 | Client ID 前缀 | `acme@device-001` |
| 4（兜底）| 无租户信息 | 使用 `default` 租户 |

当多个来源同时存在时，只取最高优先级的来源。例如，同时设置了 User Property 和 Username 前缀，以 User Property 为准。

---

## 租户管理

### 创建租户

在使用租户之前，需要先通过 Dashboard 或 API 创建对应的租户。客户端连接时若指定的租户名不存在，连接将被拒绝。

通过 RobustMQ Dashboard 创建租户：

1. 访问 [http://demo.robustmq.com/](http://demo.robustmq.com/)
2. 导航到**系统管理** -> **租户管理**
3. 点击**新建租户**，填写租户名称
4. 点击**确认**完成创建

### 查看租户列表

在 Dashboard 的**系统管理** -> **租户管理**页面，可以查看所有已创建的租户及其状态。

---

## 各功能与租户的关系

### 连接和会话

每个租户的客户端连接和会话相互独立，不同租户的 client_id 可以相同而不会冲突。

```bash
# 租户 acme 下的 device-001
mqttx conn -i "acme@device-001" -h 127.0.0.1 -p 1883

# 租户 beta 下同名的 device-001，两者互不影响
mqttx conn -i "beta@device-001" -h 127.0.0.1 -p 1883
```

### 主题发布与订阅

主题按租户隔离，不同租户的同名主题互不干扰：

```bash
# 租户 acme 发布到 sensor/temp
mqttx pub -i "acme@pub-1" -u "acme@user" -t "sensor/temp" -m "25.3"

# 租户 beta 订阅同名主题，不会收到 acme 的消息
mqttx sub -i "beta@sub-1" -u "beta@user" -t "sensor/temp"
```

### 自动订阅

自动订阅规则按租户配置，只对该租户下的客户端生效。详见 [自动订阅](./AutoSubscription.md)。

### 共享订阅

共享订阅的分组在租户内隔离，不同租户的同名分组相互独立。详见 [共享订阅](./SharedSubscription.md)。

### 访问控制（ACL）

ACL 规则和黑名单均按租户隔离配置。详见安全相关文档。

---

## 注意事项

- **租户必须先创建**：连接前确认目标租户已存在，否则连接会被拒绝并返回错误
- **`@` 为保留分隔符**：如果真实的 client_id 或 username 本身包含 `@`，只有第一个 `@` 之前的部分会被识别为租户名，其余部分作为真实值
- **默认租户**：未携带租户信息的连接归属 `default` 租户，该租户默认存在，无需手动创建
- **MQTT v3 限制**：MQTT v3.1/v3.1.1 不支持 User Property，只能通过 Username 或 Client ID 前缀传递租户信息
