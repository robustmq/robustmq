# 集群管理命令

## 集群管理 (`cluster`)

集群配置管理。

### 基本语法
```bash
robust-ctl cluster [选项] <操作>
```

### 选项
- `--server, -s <服务器>`: 服务器地址 (默认: 127.0.0.1:8080)

### 配置管理 (`config`)

```bash
# 获取集群配置
robust-ctl cluster config get
```

---

## 使用示例

```bash
# 查看集群配置帮助
robust-ctl cluster --help
robust-ctl cluster config --help

# 连接到指定服务器获取配置
robust-ctl cluster --server 192.168.1.100:8080 config get

# 获取本地集群配置
robust-ctl cluster config get
```

---

## 功能说明

集群管理模块主要用于：

1. **配置查看**: 获取当前集群的配置信息
2. **集群状态**: 了解集群的运行状态
3. **配置管理**: 查看和管理集群级别的配置参数

---

## 注意事项

- 集群配置操作通常需要管理员权限
- 确保网络连接到正确的集群管理服务
- 配置信息以 JSON 格式返回，便于解析和处理