# RobustMQ CLI 通用指南

## 概述

`robust-ctl` 是 RobustMQ 的命令行管理工具，基于 Rust clap 库构建，提供对 MQTT 代理、集群配置和日志引擎的管理功能。

## 安装和构建

```bash
# 构建项目
cargo build --release

# 运行工具
./target/release/robust-ctl --help
```

## 基本语法

```bash
robust-ctl [选项] <命令>
```

## 全局选项

- `--help, -h`: 显示帮助信息
- `--version, -V`: 显示版本信息

---

## 文档导航

- 🔧 **[MQTT 代理管理](CLI_MQTT.md)** - MQTT 代理相关的所有命令
- 🏗️ **[集群管理](CLI_CLUSTER.md)** - 集群配置管理命令
- 📝 **[日志引擎管理](CLI_JOURNAL.md)** - 日志引擎相关命令

## 快速命令索引

### MQTT 代理管理命令
```bash
robust-ctl mqtt [选项] <操作>
```

主要功能：
- 会话管理 (`session`)
- 订阅管理 (`subscribes`) 
- 用户管理 (`user`)
- 访问控制列表 (`acl`)
- 黑名单管理 (`blacklist`)
- 客户端连接管理 (`client`)
- 主题管理 (`topic`)
- 主题重写规则 (`topic-rewrite`)
- 连接器管理 (`connector`)
- 模式管理 (`schema`)
- 自动订阅规则 (`auto-subscribe`)
- 发布消息 (`publish`)
- 订阅消息 (`subscribe`)
- 可观测性功能

### 集群管理命令
```bash
robust-ctl cluster [选项] <操作>
```

主要功能：
- 配置管理 (`config`)

### 日志引擎管理命令
```bash
robust-ctl journal [选项]
```

主要功能：
- 状态查看 (目前处于开发中)

## 快速开始

```bash
# 查看工具帮助
robust-ctl --help

# 查看 MQTT 命令帮助
robust-ctl mqtt --help

# 列出所有用户
robust-ctl mqtt user list

# 获取集群配置
robust-ctl cluster config get
```

## 获取帮助

每个命令都支持 `--help` 参数来获取详细帮助：

```bash
robust-ctl --help                    # 工具总体帮助
robust-ctl mqtt --help               # MQTT 模块帮助
robust-ctl mqtt user --help          # 用户管理帮助
robust-ctl cluster --help            # 集群管理帮助
robust-ctl journal --help            # 日志引擎帮助
```

---

## 使用示例

### 基础操作示例

```bash
# 1. 查看帮助
robust-ctl --help
robust-ctl mqtt --help
robust-ctl mqtt user --help

# 2. 连接到指定服务器
robust-ctl mqtt --server 192.168.1.100:8080 session list

# 3. 用户管理
robust-ctl mqtt user create --username testuser --password testpass
robust-ctl mqtt user list
robust-ctl mqtt user delete --username testuser

# 4. ACL 管理
robust-ctl mqtt acl create \
  --cluster-name mycluster \
  --resource-type ClientId \
  --resource-name client001 \
  --topic "sensor/+" \
  --ip "192.168.1.0/24" \
  --action Publish \
  --permission Allow

# 5. 发布和订阅
robust-ctl mqtt publish \
  --username testuser \
  --password testpass \
  --topic "test/topic" \
  --qos 1

robust-ctl mqtt subscribe \
  --username testuser \
  --password testpass \
  --topic "test/topic" \
  --qos 1

# 6. 集群配置
robust-ctl cluster config get
```

### 高级管理示例

```bash
# 1. 黑名单管理
robust-ctl mqtt blacklist create \
  --cluster-name mycluster \
  --blacklist-type ClientId \
  --resource-name malicious_client \
  --end-time 1735689600 \
  --desc "因可疑活动被阻止"

# 2. 主题重写规则
robust-ctl mqtt topic-rewrite create \
  --action redirect \
  --source-topic "old/topic/+" \
  --dest-topic "new/topic/+" \
  --regex "old/(.*)"

# 3. 自动订阅规则
robust-ctl mqtt auto-subscribe create \
  --topic "system/alerts/+" \
  --qos 2 \
  --no-local \
  --retain-as-published

# 4. 模式管理
robust-ctl mqtt schema create \
  --schema-name temperature_schema \
  --schema-type json \
  --schema '{"type":"object","properties":{"temp":{"type":"number"}}}' \
  --desc "温度传感器数据模式"
```

---

## 错误处理

工具提供详细的错误信息和帮助：

- 使用 `--help` 或 `-h` 获取命令帮助
- 检查服务器连接状态
- 验证参数格式和权限
- 查看详细的错误消息和建议

---

## 配置

工具默认连接到 `127.0.0.1:8080`。您可以使用 `--server` 参数指定不同的服务器地址。

---

## 注意事项

1. **权限要求**: 某些操作需要管理员权限
2. **网络连接**: 确保能够访问 RobustMQ 服务器
3. **参数验证**: 工具会验证参数格式和有效性
4. **交互模式**: 发布和订阅命令支持交互模式
5. **分页支持**: 列表命令支持分页显示 (默认每页 10000 条)

---

## 版本信息

- 工具版本: 0.0.1
- 作者: RobustMQ Team
- 基于: Rust clap 库

---

## 更多信息

更多详细信息请参考 [RobustMQ 官方文档](https://robustmq.com)。
