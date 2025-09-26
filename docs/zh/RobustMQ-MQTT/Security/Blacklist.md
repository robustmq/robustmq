# 黑名单

黑名单是 RobustMQ 安全机制中的重要组成部分，用于阻止恶意客户端或可疑 IP 地址的连接和访问。黑名单功能提供了灵活的限制机制，支持多种匹配模式和时间限制。

## 概述

RobustMQ 的黑名单系统具有以下特点：

- **多维度限制**：支持基于用户名、客户端 ID、IP 地址的黑名单控制
- **模式匹配**：支持精确匹配、正则表达式匹配和 CIDR 网段匹配
- **时间限制**：支持设置黑名单的过期时间，自动解除限制
- **优先级高**：黑名单检查优先于 ACL 权限检查
- **高性能缓存**：黑名单规则缓存在内存中，确保快速访问控制
- **动态管理**：支持动态添加、删除和更新黑名单规则

## 黑名单检查流程

RobustMQ 的黑名单检查在权限验证的早期阶段进行，检查顺序如下：

1. **用户黑名单检查**：检查用户名是否在黑名单中
2. **用户模式匹配检查**：使用正则表达式匹配用户名
3. **客户端 ID 黑名单检查**：检查客户端 ID 是否在黑名单中
4. **客户端 ID 模式匹配检查**：使用正则表达式匹配客户端 ID
5. **IP 地址黑名单检查**：检查 IP 地址是否在黑名单中
6. **IP 网段匹配检查**：使用 CIDR 格式匹配 IP 地址范围

## 黑名单类型

### 基础类型

| 类型     | 说明             | 匹配方式 |
| -------- | ---------------- | -------- |
| User     | 用户名黑名单     | 精确匹配 |
| ClientId | 客户端 ID 黑名单 | 精确匹配 |
| IP       | IP 地址黑名单    | 精确匹配 |

### 高级类型

| 类型          | 说明               | 匹配方式   |
| ------------- | ------------------ | ---------- |
| UserMatch     | 用户名模式匹配     | 正则表达式 |
| ClientIdMatch | 客户端 ID 模式匹配 | 正则表达式 |
| IPCIDR        | IP 网段匹配        | CIDR 格式  |

## 配置黑名单

### 使用命令行工具

#### 添加黑名单

```bash
# 添加用户黑名单
robust-ctl mqtt blacklist create \
  --cluster-name robustmq-cluster \
  --blacklist-type User \
  --resource-name malicious_user \
  --end-time "2024-12-31 23:59:59" \
  --desc "恶意用户，违反使用条款"

# 添加客户端 ID 黑名单
robust-ctl mqtt blacklist create \
  --cluster-name robustmq-cluster \
  --blacklist-type ClientId \
  --resource-name bad_client_001 \
  --end-time "2024-06-30 12:00:00" \
  --desc "异常客户端，频繁发送垃圾消息"

# 添加 IP 地址黑名单
robust-ctl mqtt blacklist create \
  --cluster-name robustmq-cluster \
  --blacklist-type Ip \
  --resource-name "192.168.1.100" \
  --end-time "2024-03-31 00:00:00" \
  --desc "可疑 IP，尝试暴力破解"

# 添加用户名模式匹配黑名单
robust-ctl mqtt blacklist create \
  --cluster-name robustmq-cluster \
  --blacklist-type UserMatch \
  --resource-name "test.*" \
  --end-time "2024-12-31 23:59:59" \
  --desc "测试用户禁止访问生产环境"

# 添加客户端 ID 模式匹配黑名单
robust-ctl mqtt blacklist create \
  --cluster-name robustmq-cluster \
  --blacklist-type ClientIdMatch \
  --resource-name "bot_.*" \
  --end-time "2024-12-31 23:59:59" \
  --desc "禁止机器人客户端"

# 添加 IP 网段黑名单
robust-ctl mqtt blacklist create \
  --cluster-name robustmq-cluster \
  --blacklist-type IPCIDR \
  --resource-name "10.0.0.0/24" \
  --end-time "2024-12-31 23:59:59" \
  --desc "禁止特定网段访问"
```

#### 查询黑名单

```bash
# 查询所有黑名单
robust-ctl mqtt blacklist list
```

#### 删除黑名单

```bash
# 删除特定黑名单
robust-ctl mqtt blacklist delete \
  --cluster-name robustmq-cluster \
  --blacklist-type User \
  --resource-name malicious_user
```

### 使用 HTTP API

#### 添加黑名单

```bash
curl -X POST http://localhost:8080/api/mqtt/blacklist/create \
  -H "Content-Type: application/json" \
  -d '{
    "blacklist_type": "User",
    "resource_name": "malicious_user",
    "end_time": 1735689599,
    "desc": "恶意用户，违反使用条款"
  }'
```

#### 查询黑名单

```bash
# 查询所有黑名单
curl -X POST http://localhost:8080/api/mqtt/blacklist/list \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 10,
    "page": 1
  }'

# 查询特定类型黑名单（使用过滤参数）
curl -X POST http://localhost:8080/api/mqtt/blacklist/list \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 10,
    "page": 1,
    "filter_field": "blacklist_type",
    "filter_values": ["User"],
    "exact_match": "true"
  }'
```

#### 删除黑名单

```bash
curl -X POST http://localhost:8080/api/mqtt/blacklist/delete \
  -H "Content-Type: application/json" \
  -d '{
    "blacklist_type": "User",
    "resource_name": "malicious_user"
  }'
```

### 直接操作存储

如果使用 Placement Center 存储，可以通过内部 API 操作：

```rust
// 添加黑名单
let blacklist = MqttAclBlackList {
    blacklist_type: MqttAclBlackListType::User,
    resource_name: "malicious_user".to_string(),
    end_time: now_second() + 86400, // 24小时后过期
    desc: "恶意用户".to_string(),
};
cache_manager.add_blacklist(blacklist);
```

## 故障排除

### 连接被黑名单阻止

1. **检查黑名单规则**：确认客户端是否匹配黑名单规则
2. **检查时间设置**：确认黑名单是否已过期
3. **验证匹配模式**：检查正则表达式或 CIDR 格式是否正确
4. **查看详细日志**：启用调试日志查看详细的匹配过程

### 黑名单不生效

1. **缓存刷新**：手动刷新黑名单缓存
2. **配置同步**：确认配置已同步到所有节点
3. **规则格式检查**：验证正则表达式或 CIDR 格式
4. **时间检查**：确认黑名单未过期

### 性能问题

1. **优化正则表达式**：简化复杂的匹配模式
2. **清理过期规则**：定期清理过期的黑名单
3. **监控内存使用**：检查黑名单缓存的内存占用
4. **批量操作优化**：使用批量 API 减少操作次数

## 常见问题

### Q: 黑名单的优先级是什么？

A: 黑名单检查优先于所有其他权限检查，包括超级用户权限。即使是超级用户，如果在黑名单中也会被拒绝访问。

### Q: 正则表达式匹配的性能如何？

A: RobustMQ 对正则表达式匹配进行了优化，使用编译缓存和分组匹配。建议避免过于复杂的正则表达式以获得最佳性能。

### Q: 如何实现临时黑名单？

A: 设置合适的过期时间即可实现临时黑名单：

```bash
# 1小时临时黑名单
robust-ctl mqtt blacklist create \
  --cluster-name robustmq-cluster \
  --blacklist-type User \
  --resource-name temp_user \
  --end-time "$(date -d '+1 hour' '+%Y-%m-%d %H:%M:%S')" \
  --desc "临时限制1小时"
```

### Q: 黑名单是否支持通配符？

A: 精确匹配类型（User、ClientId、IP）不支持通配符，但可以使用模式匹配类型（UserMatch、ClientIdMatch、IPCIDR）实现类似功能。

### Q: 如何批量清理过期黑名单？

A: 可以使用管理 API 或定时任务清理：

```bash
# 清理过期黑名单的脚本
#!/bin/bash
current_time=$(date +%s)
robust-ctl mqtt blacklist list | \
jq -r ".[] | select(.end_time < $current_time) | \"\\(.blacklist_type) \\(.resource_name)\"" | \
while read blacklist_type resource_name; do
    robust-ctl mqtt blacklist delete \
      --cluster-name robustmq-cluster \
      --blacklist-type "$blacklist_type" \
      --resource-name "$resource_name"
done
```

### Q: 黑名单规则是否会自动同步到集群？

A: 黑名单规则会通过定期缓存同步机制同步到集群中的所有节点。与用户和 ACL 规则不同，黑名单不支持实时推送同步，而是依赖每个 broker 节点每秒从 meta-service 拉取最新的黑名单数据来更新本地缓存。
