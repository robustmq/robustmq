## 概述
RobustMQ 长期预计支持多种消息队列协议。当前已支持 MQTT 协议，即：RobustMQ MQTT.

## RobustMQ MQTT

### 部署模式
RobustMQ MQTT 单机和集群两种部署模式。
- 单机模式：启动单机模式的 MQTT Server，其中 Placement Center 和 MQTT Server 都是单机运行。
- 集群模式：启动集群模式的 MQTT Server，其中 Placement Center 和 MQTT Server 都是多节点集群模式运行。
其中 Placement Center 默认三节点，MQTT Broker 节点数量不限制。(该部分仅支持部署，命令行暂时还未支持)

## 运行方式
1. Cargo 运行： 下载源代码，然后执行 cargo run 命令运行 MQTT Server。该方式适用于开发调试。
2. 二进制包运行：下载或编译二进制包，然后执行二进制包运行 MQTT Server。该方式适用于生产环境。
3. Docker 运行：即下载或编译 Docker 镜像，然后执行 Docker 镜像运行 MQTT Server。该方式适用于生产环境。
4. K8s 运行：即在 K8s 集群中运行 MQTT Server。该方式适用于生产环境。

> 建议：在开发调试阶段，我们一般用 Cargo 运行。在生产环境我们一般推荐用 docker 或 K8s 方式运行，因为这样可以方便的进行扩容和缩容。同时我们也支持二进制包运行。

## 一键快速启动
RobustMQ 提供了一键快速启动脚本，可以快速启动单机MQTT Server，进行体验和测试。
```bash
# 下载脚本
curl -fsSL -o install.sh https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh
chmod +x install.sh

# 运行安装
./install.sh
```

介于国内网速问题，可以拉取仓库代码，通过仓库来进行脚本编译
```bash
./scripts/build.sh
🔨 RobustMQ Build Script

[2025-08-27 20:56:03] 🚀 Build Configuration
[2025-08-27 20:56:03] ℹ️  Version: v0.1.24-82-ge4f27a63
[2025-08-27 20:56:03] ℹ️  Component: server
[2025-08-27 20:56:03] ℹ️  Platform: darwin-arm64
[2025-08-27 20:56:03] ℹ️  Build Type: release
[2025-08-27 20:56:03] ℹ️  Output Directory: /Users/xxxxx/other/code/rustRoverProject/robustmq/build
[2025-08-27 20:56:03] ℹ️  Parallel Builds: true

[2025-08-27 20:56:03] 🚀 Checking dependencies...
[2025-08-27 20:56:03] 🚀 Building for platform: darwin-arm64
[2025-08-27 20:56:03] 🚀 Building server component for darwin-arm64
[2025-08-27 20:56:03] ℹ️  Compiling Rust binaries for aarch64-apple-darwin...
[2025-08-27 20:56:03] ℹ️  Running: cargo build --target aarch64-apple-darwin --release
    Finished `release` profile [optimized] target(s) in 0.36s
[2025-08-27 20:56:03] ℹ️  Creating tarball for darwin-arm64...
[2025-08-27 20:56:04] ✅ Server component built successfully: robustmq-v0.1.24-82-ge4f27a63-darwin-arm64.tar.gz

[2025-08-27 20:56:04] ✅ Build completed successfully!
[2025-08-27 20:56:04] ℹ️  Output directory: /Users/xxxxx/other/code/rustRoverProject/robustmq/build
[2025-08-27 20:56:04] ℹ️  Generated packages:
  • robustmq-0.1.20.tar.gz
  • robustmq-v0.1.24-82-ge4f27a63-darwin-arm64.tar.gz

````
执行完成后会得到如下结构的二进制包
``` shell
(base) ➜  build git:(build) ✗ tree robustmq-v0.1.24-82-ge4f27a63-darwin-arm64
robustmq-v0.1.24-82-ge4f27a63-darwin-arm64
├── bin
│   ├── robust-bench
│   ├── robust-ctl
│   └── robust-server
├── config
│   ├── certs
│   │   ├── ca.pem
│   │   ├── cert.pem
│   │   └── key.pem
│   ├── server-tracing.toml
│   ├── server.toml
│   ├── server.toml.template
│   ├── version.ini
│   └── version.txt
├── docs
├── libs
│   ├── broker-server
│   ├── cli-bench
│   └── cli-command
└── package-info.txt

6 directories, 15 files
```
接下来我们只需要执行
```bash
cd robustmq-v0.1.24-82-ge4f27a63-darwin-arm64
./bin/robust-server start config/server.toml

Config: config/server.toml
Starting RobustMQ broker server...
✅ RobustMQ broker started successfully.
📁 Log file location: /Users/xxxx/other/code/rustRoverProject/robustmq/build/robustmq-v0.1.24-82-ge4f27a63-darwin-arm64/logs/robustmq.log
📝 To view logs in real-time: tail -f "/Users/xxxx/other/code/rustRoverProject/robustmq/build/robustmq-v0.1.24-82-ge4f27a63-darwin-arm64/logs/robustmq.log"
🔍 To check service status: ps -ef | grep broker-server
💡 Log directory: /Users/xxxx/other/code/rustRoverProject/robustmq/build/robustmq-v0.1.24-82-ge4f27a63-darwin-arm64/logs
```
即可启动单机的 MQTT Server 进行体验和测试。

### 快速启动 - 当前版本有变动，正在重新修正
- [编译二进制安装包【可选】](mqtt/Build.md)
- [二进制运行-单机模式](mqtt/Run-Standalone-Mode.md)
- [二进制运行-集群模式](mqtt/Run-Cluster-Mode.md)
- [Docker 运行](mqtt/Run-Docker-Mode.md)
- [K8s 运行](mqtt/Run-K8S-Mode.md)
