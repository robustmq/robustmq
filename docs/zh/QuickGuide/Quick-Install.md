# 快速安装指南

本指南帮助您快速安装并启动 RobustMQ。

## 目录

- [安装方式](#安装方式)
- [验证安装](#验证安装)
- [常见问题](#常见问题)

---

## 安装方式

### 方式一：自动安装脚本（推荐）

```bash
# 一键安装最新版本（默认安装到 ~/robustmq-{版本号}，如 ~/robustmq-v0.3.0）
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
```

脚本会把完整包解压到带版本号的目录（默认 `~/robustmq-{版本号}`，如 `~/robustmq-v0.3.0`），并将该目录下的 `bin/` 加入 PATH。安装完成后，以下三个命令即可全局使用：

| 命令 | 说明 |
|------|------|
| `robust-server` | 启动和管理 RobustMQ 服务 |
| `robust-ctl` | 命令行管理工具（查看集群状态、管理 Topic 等）|
| `robust-bench` | 压测工具 |

#### 常用安装选项

```bash
# 安装指定版本
VERSION=v0.3.0 curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash

# 安装到指定目录（整个包会解压到该目录，bin/ 子目录加入 PATH）
INSTALL_DIR=/opt/robustmq curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash

# 查看所有安装选项
wget https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh
chmod +x install.sh && ./install.sh --help
```

安装后的目录结构：
```
~/robustmq-v0.3.0/    ← 实际安装目录（含版本号）
  bin/
    robust-server
    robust-ctl
    robust-bench
  libs/
    broker-server     ← 实际二进制（脚本内部调用，不直接暴露）
    cli-command
    cli-bench
  config/
    server.toml       ← 配置文件
  logs/               ← 运行时日志（自动创建）

~/robustmq  ->  ~/robustmq-v0.3.0   ← 稳定短链接
```

PATH 中加入的是 `~/robustmq/bin`（短链接路径），升级新版本时只需更新软链接，PATH 配置无需改动。

**安装脚本选项：**

| 选项 | 说明 |
|------|------|
| `--version VERSION` | 安装指定版本（默认：最新）|
| `--dir DIRECTORY` | 安装目录（默认：自动检测）|
| `--silent` | 静默安装 |
| `--force` | 强制重新安装 |
| `--dry-run` | 预览安装内容（不实际安装）|

---

### 方式二：手动下载二进制包

访问 [GitHub Releases](https://github.com/robustmq/robustmq/releases) 下载对应平台的包：

```bash
# 以 Linux x86_64 为例，请替换为实际版本号和平台
wget https://github.com/robustmq/robustmq/releases/latest/download/robustmq-v0.3.0-linux-amd64.tar.gz

tar -xzf robustmq-v0.3.0-linux-amd64.tar.gz
cd robustmq-v0.3.0-linux-amd64

# 启动服务
./bin/robust-server start
```

**当前支持的平台：**

| 平台 | 说明 |
|------|------|
| `linux-amd64` | Linux x86_64 |
| `linux-arm64` | Linux ARM64 |
| `darwin-arm64` | macOS Apple Silicon (M1/M2/M3) |

---

### 方式三：Docker

```bash
# 拉取最新镜像
docker pull ghcr.io/robustmq/robustmq:latest

# 启动容器
docker run -d \
  -p 1883:1883 \
  -p 8080:8080 \
  --name robustmq \
  ghcr.io/robustmq/robustmq:latest

# 指定版本
docker pull ghcr.io/robustmq/robustmq:v0.3.0
```

镜像支持 `linux/amd64` 和 `linux/arm64` 多架构，托管在 [GitHub Container Registry](https://github.com/robustmq/robustmq/pkgs/container/robustmq)。

---

### 方式四：从源码构建

```bash
git clone https://github.com/robustmq/robustmq.git
cd robustmq

# 编译并运行
cargo run --package cmd --bin broker-server -- --conf config/server.toml
```

---

## 验证安装

### 启动服务

```bash
# 前台启动
robust-server start

# 后台启动
robust-server start &

# 使用自定义配置文件启动
robust-server start /path/to/server.toml
```

启动成功后，输出类似：

```
[INFO] RobustMQ Broker starting...
[INFO] MQTT server listening on 0.0.0.0:1883
[INFO] Admin server listening on 0.0.0.0:8080
[INFO] Broker started successfully
```

### 查看集群状态

```bash
robust-ctl status
```

### 测试 MQTT 连接

```bash
# 发布消息
mqttx pub -h localhost -p 1883 -t "test/topic" -m "Hello RobustMQ!"

# 订阅消息
mqttx sub -h localhost -p 1883 -t "test/topic"
```

### Web 管理控制台

访问 `http://localhost:8080` 打开 Web 管理界面。

---

## 常见问题

**Q: 安装脚本失败怎么办？**

检查网络连接，或手动下载二进制包（方式二）。

**Q: 权限不足怎么办？**

```bash
# 安装到用户目录，无需 sudo
INSTALL_DIR=$HOME/.local/bin curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
```

**Q: 如何卸载？**

```bash
# 删除软链接和安装目录
rm ~/robustmq
rm -rf ~/robustmq-v0.3.0

# 同时清理 PATH 配置（从 ~/.bashrc 或 ~/.zshrc 中删除 ~/robustmq/bin 那行）
```

**Q: 如何升级到新版本？**

直接运行安装脚本即可，会自动安装新版本并将软链接 `~/robustmq` 指向新版本目录：

```bash
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
```

**Q: 端口被占用怎么办？**

```bash
# 检查 1883 端口（MQTT）
netstat -tlnp | grep 1883

# 检查 8080 端口（Admin）
netstat -tlnp | grep 8080
```

**Q: 如何修改配置？**

```bash
# 编辑配置文件
vim config/server.toml

# 使用自定义配置启动
robust-server start /path/to/server.toml
```

---

## 下一步

- [体验 MQTT 功能](Experience-MQTT.md)
- [什么是 RobustMQ](../OverView/What-is-RobustMQ.md)

## 获取帮助

- **官方文档**：https://robustmq.com
- **GitHub Issues**：https://github.com/robustmq/robustmq/issues
- **GitHub Discussions**：https://github.com/robustmq/robustmq/discussions
- **Discord**：https://discord.gg/sygeGRh5
