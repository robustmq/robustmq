# Pprof 性能分析工具使用指南

## 概述

Pprof 是 RobustMQ 内置的性能分析工具，用于生成应用程序的性能火焰图，帮助开发者识别性能瓶颈和优化机会。

## 配置

在 `config/server.toml` 文件中添加以下配置：

```toml
[p_prof]
enable = true      # 启用 pprof 功能
port = 6777        # HTTP 服务端口
frequency = 1000   # 采样频率 (Hz)
```

### 配置参数说明

- `enable`: 是否启用 pprof 监控，默认为 `false`
- `port`: HTTP 服务器监听端口，默认为 `6060`
- `frequency`: 性能采样频率，单位 Hz，默认为 `100`

## 使用方法

### 1. 启动服务

确保配置文件中 `enable = true`，然后启动 RobustMQ 服务：

```bash
./bin/robust-server start
```

服务启动后会看到类似日志：
```
Pprof HTTP Server started successfully, listening port: 6777
```

### 2. 生成火焰图

在浏览器中访问：
```
http://127.0.0.1:6777/flamegraph
```

系统会返回 SVG 格式的性能火焰图。

### 3. 分析火焰图

- **宽度**：函数调用的时间占比
- **高度**：调用栈的深度
- **颜色**：不同函数的区分标识
- **热点**：宽度较大的区域表示性能瓶颈

