# Pprof 性能分析工具使用指南

## 概述

Pprof 是 RobustMQ 内置的性能分析工具，用于生成应用程序的性能火焰图，帮助开发者识别性能瓶颈和优化机会。它没有独立的 HTTP 端口，采集开关由 `[runtime]` 的 `pprof_enable` 控制，火焰图通过 Admin HTTP API（复用 `http_port`）暴露。

## 配置

在 `config/server.toml` 文件中添加以下配置：

```toml
[runtime]
pprof_enable = true   # 启用 pprof 采集，默认为 false
```

## 使用方法

### 1. 启动服务

确保配置文件中 `runtime.pprof_enable = true`，然后启动 RobustMQ 服务：

```bash
./bin/robust-server start
```

### 2. 生成火焰图

在浏览器中访问（`{http_port}` 替换为 `server.toml` 中配置的 `http_port`，默认为 `58080`）：

```
http://127.0.0.1:{http_port}/debug/pprof/flamegraph
```

系统会返回 SVG 格式的性能火焰图。若 `pprof_enable` 未开启，该接口会返回提示文本而非火焰图。

### 3. 分析火焰图

- **宽度**：函数调用的时间占比
- **高度**：调用栈的深度
- **颜色**：不同函数的区分标识
- **热点**：宽度较大的区域表示性能瓶颈
