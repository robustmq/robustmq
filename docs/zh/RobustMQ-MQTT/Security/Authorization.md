# 授权

访问控制列表（ACL）是 MQTT 安全机制的重要组成部分，RobustMQ 提供了灵活且精细的权限控制系统，可以基于用户、客户端 ID、主题和 IP 地址等多个维度进行访问控制。

## 概述

RobustMQ 的 ACL 授权系统具有以下特点：

- **细粒度权限控制**：支持发布、订阅、保留消息等不同操作的权限控制
- **多维度匹配**：支持基于用户名、客户端 ID、主题、IP 地址的权限匹配
- **灵活的权限策略**：支持 Allow（允许）和 Deny（拒绝）两种权限类型
- **通配符支持**：支持使用通配符进行主题和 IP 地址匹配
- **超级用户绕过**：超级用户可以绕过所有 ACL 检查
- **高性能缓存**：ACL 规则缓存在内存中，确保高性能访问控制

## 权限检查流程

RobustMQ 的权限检查按以下顺序进行：

1. **超级用户检查**：如果是超级用户，直接允许所有操作
2. **黑名单检查**：检查用户、客户端 ID 或 IP 是否在黑名单中
3. **ACL 规则检查**：按优先级检查 ACL 规则，找到匹配的拒绝规则则拒绝访问
4. **保留消息权限检查**：如果是保留消息，额外检查 Retain 权限
5. **默认策略**：如果没有匹配的拒绝规则，则允许访问

## ACL 权限类型

### 操作类型

| 操作类型  | 数值 | 说明                           |
| --------- | ---- | ------------------------------ |
| All       | 0    | 所有操作（发布、订阅、保留等） |
| Subscribe | 1    | 订阅主题                       |
| Publish   | 2    | 发布消息                       |
| PubSub    | 3    | 发布和订阅                     |
| Retain    | 4    | 保留消息                       |
| Qos       | 5    | QoS 相关操作                   |

### 权限类型

| 权限类型 | 数值 | 说明     |
| -------- | ---- | -------- |
| Allow    | 1    | 允许访问 |
| Deny     | 0    | 拒绝访问 |

### 资源类型

| 资源类型 | 说明                     |
| -------- | ------------------------ |
| User     | 基于用户名的权限控制     |
| ClientId | 基于客户端 ID 的权限控制 |

## 配置 ACL 规则

### 使用命令行工具

#### 创建 ACL 规则

```bash
# 允许用户 testuser 发布到 test/# 主题
robust-ctl mqtt acl create \
  --cluster-name robustmq-cluster \
  --resource-type User \
  --resource-name testuser \
  --topic "test/#" \
  --ip "*" \
  --action Publish \
  --permission Allow

# 拒绝客户端 bad_client 订阅所有主题
robust-ctl mqtt acl create \
  --cluster-name robustmq-cluster \
  --resource-type ClientId \
  --resource-name bad_client \
  --topic "*" \
  --ip "*" \
  --action Subscribe \
  --permission Deny

# 允许特定 IP 地址的用户访问所有主题
robust-ctl mqtt acl create \
  --cluster-name robustmq-cluster \
  --resource-type User \
  --resource-name admin \
  --topic "*" \
  --ip "192.168.1.100" \
  --action All \
  --permission Allow
```

#### 查询 ACL 规则

```bash
# 查询所有 ACL 规则
robust-ctl mqtt acl list
```

#### 删除 ACL 规则

```bash
# 删除特定 ACL 规则
robust-ctl mqtt acl delete \
  --cluster-name robustmq-cluster \
  --resource-type User \
  --resource-name testuser \
  --topic "test/#" \
  --ip "*" \
  --action "Subscribe" \
  --permission "Allow"
```

### 使用 HTTP API

#### 创建 ACL 规则

```bash
curl -X POST http://localhost:8080/api/mqtt/acl/create \
  -H "Content-Type: application/json" \
  -d '{
    "resource_type": "User",
    "resource_name": "testuser",
    "topic": "test/#",
    "ip": "*",
    "action": "Publish",
    "permission": "Allow"
  }'
```

#### 查询 ACL 规则

```bash
# 查询所有 ACL 规则
curl -X POST http://localhost:8080/api/mqtt/acl/list \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 10,
    "page": 1
  }'

# 查询特定用户的 ACL 规则（使用过滤参数）
curl -X POST http://localhost:8080/api/mqtt/acl/list \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 10,
    "page": 1,
    "filter_field": "resource_name",
    "filter_values": ["testuser"],
    "exact_match": "true"
  }'
```

#### 删除 ACL 规则

```bash
curl -X POST http://localhost:8080/api/mqtt/acl/delete \
  -H "Content-Type: application/json" \
  -d '{
    "resource_type": "User",
    "resource_name": "testuser",
    "topic": "test/#",
    "ip": "*",
    "action": "Publish",
    "permission": "Allow"
  }'
```

### 直接操作数据库

如果使用 MySQL 存储后端，可以直接操作数据库：

```sql
-- 创建 ACL 规则
INSERT INTO mqtt_acl (allow, ipaddr, username, clientid, access, topic)
VALUES (1, '*', 'testuser', '', 2, 'test/#');

-- 查询 ACL 规则
SELECT * FROM mqtt_acl WHERE username = 'testuser';

-- 删除 ACL 规则
DELETE FROM mqtt_acl WHERE username = 'testuser' AND topic = 'test/#';
```

## 故障排除

### 权限被拒绝

1. **检查 ACL 规则**：确认是否有匹配的拒绝规则
2. **检查超级用户状态**：确认用户是否为超级用户
3. **检查黑名单**：确认用户、客户端 ID 或 IP 是否在黑名单中
4. **查看日志**：检查 RobustMQ 日志获取详细权限检查信息

### 权限配置不生效

1. **缓存刷新**：手动刷新 ACL 缓存
2. **配置同步**：确认配置已同步到所有节点
3. **规则优先级**：检查是否有优先级更高的规则覆盖
4. **通配符匹配**：确认通配符使用是否正确

### 性能问题

1. **优化规则数量**：减少不必要的 ACL 规则
2. **使用更精确的匹配**：避免过度使用通配符
3. **监控缓存状态**：检查 ACL 缓存命中率
4. **数据库优化**：优化 ACL 数据表的索引

## 常见问题

### Q: ACL 规则的优先级是什么？

A: RobustMQ 的 ACL 检查遵循以下优先级：

1. 超级用户绕过所有检查
2. 黑名单检查优先于 ACL
3. 拒绝规则优先于允许规则
4. 用户级规则和客户端 ID 级规则同等优先级

### Q: 如何实现主题级别的权限继承？

A: 使用通配符可以实现权限继承：

```bash
# 允许访问 sensors 下的所有子主题
robust-ctl mqtt acl create \
  --resource-type user \
  --resource-name sensor_user \
  --topic "sensors/#" \
  --action subscribe \
  --permission allow \
  --ip "*"
```

### Q: 如何调试权限问题？

A: 启用详细日志并检查权限检查过程：

```toml
[log]
level = "debug"
```

然后查看日志中的权限检查信息。

### Q: ACL 规则是否支持正则表达式？

A: 目前 ACL 规则的主题匹配使用 MQTT 标准通配符（+ 和 #），不支持正则表达式。IP 地址支持 CIDR 格式。

### Q: 如何批量导入 ACL 规则？

A: 可以使用脚本批量创建规则，或直接操作数据库进行批量导入：

```bash
#!/bin/bash
while IFS=',' read -r resource_type resource_name topic action permission ip
do
    robust-ctl mqtt acl create \
      --resource-type "$resource_type" \
      --resource-name "$resource_name" \
      --topic "$topic" \
      --action "$action" \
      --permission "$permission" \
      --ip "$ip"
done < acl_rules.csv
```
