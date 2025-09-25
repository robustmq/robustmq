# 认证

身份认证可以有效阻止非法客户端的连接，RobustMQ 提供了灵活且安全的身份认证机制，支持多种认证方式和存储后端。

## 概述

RobustMQ MQTT Broker 的认证系统具有以下特点：

- **多种认证方式**：支持明文认证、JWT 认证、HTTP 认证等
- **灵活的存储后端**：支持 Placement Center、MySQL 等存储方式
- **超级用户机制**：支持超级用户权限，可绕过 ACL 检查
- **免密登录**：支持在测试环境下免密登录配置
- **用户缓存**：提供用户信息缓存机制，提高认证性能

部分认证方式和存储后端支持正在开发中。

## 认证流程

RobustMQ 的认证流程按以下顺序进行：

1. **免密登录检查**：如果启用了 `secret_free_login`，则跳过认证直接允许连接
2. **用户名密码验证**：验证客户端提供的用户名和密码
3. **用户信息查询**：从缓存或存储层获取用户信息
4. **认证结果返回**：返回认证成功或失败

## 配置认证

### 基础配置

在 `server.toml` 配置文件中设置认证相关参数：

```toml
[mqtt_security]
# 是否启用免密登录
secret_free_login = false

[mqtt_auth_storage]
# 认证存储类型：placement | mysql
storage_type = "placement"
# MySQL 连接地址（当使用 MySQL 存储时）
mysql_addr = "mysql://username:password@localhost:3306/mqtt"
```

### 存储后端配置

#### Placement Center 存储

使用 RobustMQ 内置的 Placement Center 作为存储后端：

```toml
[mqtt_auth_storage]
storage_type = "placement"
```

#### MySQL 存储

使用 MySQL 数据库作为存储后端：

```toml
[mqtt_auth_storage]
storage_type = "mysql"
mysql_addr = "mysql://root:password@localhost:3306/mqtt"
```

需要创建相应的数据表：

```sql
CREATE TABLE `mqtt_user` (
    `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
    `username` varchar(100) DEFAULT NULL,
    `password` varchar(100) DEFAULT NULL,
    `salt` varchar(35) DEFAULT NULL,
    `is_superuser` tinyint(1) DEFAULT 0,
    `created` datetime DEFAULT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `mqtt_username` (`username`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

## 用户管理

### 创建用户

#### 使用 robust-ctl 命令行工具

```bash
# 创建普通用户
robust-ctl mqtt user create --username testuser --password testpass

# 创建超级用户
robust-ctl mqtt user create --username admin --password adminpass --is-superuser
```

#### 使用 HTTP API

```bash
# 创建普通用户
curl -X POST http://localhost:8080/api/mqtt/user/create \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "testpass",
    "is_superuser": false
  }'

# 创建超级用户
curl -X POST http://localhost:8080/api/mqtt/user/create \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "adminpass",
    "is_superuser": true
  }'

# 查询用户列表
curl -X POST http://localhost:8080/api/mqtt/user/list \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 10,
    "page": 1
  }'

# 删除用户
curl -X POST http://localhost:8080/api/mqtt/user/delete \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser"
  }'
```

#### 直接操作 MySQL

```sql
-- 创建普通用户
INSERT INTO mqtt_user (username, password, is_superuser)
VALUES ('testuser', 'testpass', 0);

-- 创建超级用户
INSERT INTO mqtt_user (username, password, is_superuser)
VALUES ('admin', 'adminpass', 1);
```

### 查询用户

```bash
# 查询所有用户
robust-ctl mqtt user list
```

### 删除用户

```bash
# 删除用户
robust-ctl mqtt user delete --username testuser
```

## 认证方式

### 明文认证

最常用的认证方式，直接验证用户名和密码：

```rust
// 客户端连接时提供用户名和密码
let login = Login {
    username: "testuser".to_string(),
    password: "testpass".to_string(),
};
```

### 免密登录

适用于测试环境，跳过用户名密码验证：

```toml
[mqtt_security]
secret_free_login = true
```

**注意：生产环境请务必关闭免密登录功能。**

## 超级用户

超级用户拥有特殊权限，可以绕过 ACL 检查，对所有主题具有完全访问权限。

### 超级用户特点

- 绕过所有 ACL 权限检查
- 可以发布和订阅任何主题
- 不受黑名单限制（除非被明确加入黑名单）
- 适用于管理员账户

### 设置超级用户

创建用户时设置 `is_superuser` 为 `true`：

```sql
INSERT INTO mqtt_user (username, password, is_superuser)
VALUES ('admin', 'admin123', 1);
```

## 常见问题

### Q: 认证失败怎么办？

A: 检查以下几个方面：

1. **用户名密码是否正确**
2. **用户是否存在**：使用 `robust-ctl mqtt user get` 查询
3. **配置是否正确**：检查 `server.toml` 中的认证配置
4. **存储层是否正常**：检查 MySQL 或 Placement Center 是否正常运行

### Q: 如何重置用户密码？

A: 可以通过以下方式重置：

```bash
# 使用命令行工具
robust-ctl mqtt user update --username testuser --password newpass

# 或直接修改数据库
UPDATE mqtt_user SET password = 'newpass' WHERE username = 'testuser';
```

### Q: 超级用户是否受黑名单限制？

A: 超级用户在 ACL 检查中会被绕过，但仍会受到黑名单检查的限制。

### Q: 如何启用调试日志？

A: 在配置文件中设置日志级别：

```toml
[log]
level = "debug"
```

### Q: 认证缓存如何更新？

A: RobustMQ 会定期同步存储层数据到缓存，也可以通过 API 手动刷新：

```bash
curl -X POST http://localhost:8080/api/mqtt/cache/refresh
```

## 故障排除

### 连接被拒绝

1. 检查用户名密码是否正确
2. 确认用户是否存在且未被禁用
3. 检查是否被加入黑名单
4. 查看 RobustMQ 日志获取详细错误信息

### 认证性能问题

1. 监控数据库连接池状态
2. 检查缓存命中率
3. 优化数据库查询性能
4. 考虑增加缓存大小

### 配置不生效

1. 确认配置文件路径正确
2. 重启 RobustMQ 服务
3. 检查配置文件语法是否正确
4. 查看启动日志确认配置加载状态
