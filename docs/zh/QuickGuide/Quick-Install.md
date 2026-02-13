# 快速安装指南

本指南将帮助您快速安装和启动 RobustMQ，包括多种安装方式和详细的验证步骤。

## 目录

- [安装方式](#安装方式)
- [验证安装](#验证安装)
- [常见问题](#常见问题)

## 安装方式

### 方式一：自动安装脚本（推荐）

#### 一键安装最新版本

```bash
# 自动安装最新版本
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash

# 启动服务
broker-server start
```

#### 指定版本安装

```bash
# 安装特定版本
VERSION=v0.1.35 curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash

# 安装到指定目录
INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
```

#### 安装选项

```bash
# 下载脚本后查看所有选项
wget https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh
chmod +x install.sh
./install.sh --help
```

**可用选项：**
- `--version VERSION`: 安装指定版本（默认：最新）
- `--dir DIRECTORY`: 安装目录（默认：自动检测）
- `--silent`: 静默安装
- `--force`: 强制安装（即使已存在）
- `--dry-run`: 预览安装（不实际安装）

### 方式二：预编译二进制包

#### 手动下载

访问 [发布页面](https://github.com/robustmq/robustmq/releases) 下载适合您平台的包：

```bash
# Linux x86_64 示例（请替换为您的平台）
wget https://github.com/robustmq/robustmq/releases/latest/download/robustmq-v0.1.35-linux-amd64.tar.gz

# 解压包
tar -xzf robustmq-v0.1.35-linux-amd64.tar.gz
cd robustmq-v0.1.35-linux-amd64

# 运行服务器
./bin/robust-server start
```

**支持的平台：** `linux-amd64`, `linux-arm64`, `darwin-amd64`, `darwin-arm64`, `windows-amd64`

### 方式三：从源码构建

```bash
# 克隆仓库
git clone https://github.com/robustmq/robustmq.git
cd robustmq

# 构建并运行
cargo run --package cmd --bin broker-server
```

### 方式四：Docker（即将推出）

```bash
# Docker 运行（即将推出）
docker run -p 1883:1883 -p 9092:9092 robustmq/robustmq:latest
```

## 验证安装

### 检查二进制文件

```bash
# 检查是否安装成功
broker-server --version
cli-command --help
cli-bench --help
```

### 启动服务器

```bash
# 启动服务器
broker-server start

# 后台启动
nohup broker-server start > broker.log 2>&1 &

# 使用配置文件启动
broker-server start config/server.toml
```

### 验证服务器状态

启动成功后，您应该看到类似以下的输出：

```
[INFO] RobustMQ Broker starting...
[INFO] MQTT server listening on 0.0.0.0:1883
[INFO] Admin server listening on 0.0.0.0:8080
[INFO] Broker started successfully
```

### 检查集群状态

```bash
# 查看集群运行状态
cli-command status
```

预期输出：
```
🚀 Checking RobustMQ status...
✅ RobustMQ Status: Online
📋 Version: RobustMQ 0.1.35
🌐 Server: 127.0.0.1:8080
```

### 连接测试

#### 使用 MQTT 客户端测试

```bash
# 使用 MQTTX 测试连接
mqttx pub -h localhost -p 1883 -t "test/topic" -m "Hello RobustMQ!"

# 订阅消息
mqttx sub -h localhost -p 1883 -t "test/topic"
```

#### 使用 Web 控制台

访问 `http://localhost:8080` 查看 Web 管理界面。

## 常见问题

### 安装问题

**Q: 安装脚本失败怎么办？**
A: 请检查网络连接，或手动下载预编译包。

**Q: 权限不足怎么办？**
A: 使用 `sudo` 或指定用户目录安装：
```bash
INSTALL_DIR=$HOME/.local/bin curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
```

**Q: 如何卸载？**
A: 删除安装的二进制文件：
```bash
rm -f /usr/local/bin/broker-server
rm -f /usr/local/bin/cli-command
rm -f /usr/local/bin/cli-bench
```

### 启动问题

**Q: 端口被占用怎么办？**
A: 检查端口占用情况：
```bash
# 检查 1883 端口
netstat -tlnp | grep 1883

# 检查 8080 端口
netstat -tlnp | grep 8080
```

**Q: 如何修改配置？**
A: 编辑配置文件：
```bash
# 编辑默认配置
vim config/server.toml

# 使用自定义配置启动
broker-server start /path/to/your/config.toml
```

### 连接问题

**Q: 无法连接到 MQTT 服务器？**
A: 检查防火墙设置和端口配置。

**Q: Web 控制台无法访问？**
A: 确认管理端口（默认 8080）是否正常启动。

## 下一步

安装完成后，您可以：

1. **体验 MQTT 功能**：查看 [MQTT 体验指南](Experience-MQTT.md)
2. **了解配置选项**：查看 [配置文档](../Configuration/BROKER.md)
3. **学习高级功能**：查看 [完整文档](../OverView/What-is-RobustMQ.md)

## 获取帮助

- **📖 [官方文档](https://robustmq.com/)** - 完整指南和 API 参考
- **🐛 [GitHub Issues](https://github.com/robustmq/robustmq/issues)** - 问题报告
- **💡 [GitHub Discussions](https://github.com/robustmq/robustmq/discussions)** - 讨论和建议
- **🎮 [Discord](https://discord.gg/sygeGRh5)** - 实时聊天和支持
