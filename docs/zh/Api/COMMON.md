# RobustMQ Admin Server HTTP API 通用指南

## 概述

RobustMQ Admin Server 是 HTTP 管理接口服务，提供对 RobustMQ 集群的全面管理功能。

- **基础地址**: `http://localhost:8080`
- **请求方法**: 主要使用 `POST` 方法
- **数据格式**: JSON
- **响应格式**: JSON

## API 文档导航

- 📋 **[集群管理 API](CLUSTER.md)** - 集群配置和状态管理
- 🔧 **[MQTT Broker API](MQTT.md)** - MQTT 代理相关的所有管理接口

---

## 通用响应格式

### 成功响应
```json
{
  "code": 0,
  "message": "success",
  "data": {...}
}
```

### 错误响应
```json
{
  "code": 500,
  "message": "error message",
  "data": null
}
```

### 分页响应格式
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [...],
    "total_count": 100
  }
}
```

---

## 通用请求参数

大多数列表查询接口支持以下通用参数：

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| `limit` | `u32` | 否 | 每页大小，默认 10000 |
| `page` | `u32` | 否 | 页码，从1开始，默认 1 |
| `sort_field` | `string` | 否 | 排序字段 |
| `sort_by` | `string` | 否 | 排序方式：asc/desc |
| `filter_field` | `string` | 否 | 过滤字段 |
| `filter_values` | `array` | 否 | 过滤值列表 |
| `exact_match` | `string` | 否 | 精确匹配：true/false |

### 分页参数示例
```json
{
  "limit": 20,
  "page": 1,
  "sort_field": "create_time",
  "sort_by": "desc",
  "filter_field": "status",
  "filter_values": ["active"],
  "exact_match": "false"
}
```

---

## 基础接口

### 服务状态查询
- **接口**: `GET /`
- **描述**: 获取服务版本信息
- **请求参数**: 无
- **响应示例**:
```json
"RobustMQ v0.1.31"
```

---

## 错误码说明

| 错误码 | 说明 |
|--------|------|
| 0 | 请求成功 |
| 400 | 请求参数错误 |
| 401 | 未授权 |
| 403 | 禁止访问 |
| 404 | 资源不存在 |
| 500 | 服务器内部错误 |

---

## 使用示例

### 基本请求示例
```bash
# 获取服务版本
curl -X GET http://localhost:8080/

# 带分页的列表查询
curl -X POST http://localhost:8080/mqtt/user/list \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 10,
    "page": 1,
    "sort_field": "username",
    "sort_by": "asc"
  }'
```

### 错误处理示例
```bash
# 当请求失败时，会返回错误信息
{
  "code": 400,
  "message": "Invalid parameter: username is required",
  "data": null
}
```

---

## 注意事项

1. **请求方法**: 除了根路径 `/` 使用 GET 方法外，所有其他接口都使用 POST 方法
2. **请求体**: 即使是查询操作，也需要发送 JSON 格式的请求体
3. **时间格式**: 
   - 输入时间使用 Unix 时间戳（秒）
   - 输出时间使用本地时间格式字符串 "YYYY-MM-DD HH:MM:SS"
4. **分页**: 页码 `page` 从 1 开始计数
5. **配置验证**: 创建资源时会验证配置格式的正确性
6. **权限控制**: 建议在生产环境中添加适当的认证和授权机制
7. **错误处理**: 所有错误都会返回详细的错误信息，便于调试
8. **内容类型**: 请求必须设置 `Content-Type: application/json` 头部

---

## 开发和调试

### 启动服务
```bash
# 启动 admin-server
cargo run --bin admin-server

# 或者使用已编译的二进制文件
./target/release/admin-server
```

### 测试连接
```bash
# 测试服务是否正常运行
curl -X GET http://localhost:8080/
```

### 日志查看
服务运行时会输出详细的日志信息，包括：
- 请求路径和参数
- 响应状态和数据
- 错误信息和堆栈跟踪

---

*文档版本: v3.0*  
*最后更新: 2024-01-01*  
*基于代码版本: RobustMQ Admin Server v0.1.31*
