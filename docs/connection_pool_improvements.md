# gRPC 客户端连接池优化文档

## 概述

针对连接池超时和连接管理问题，我们对 `grpc-clients` 模块进行了全面优化，主要改进了 MQTT Broker 连接池的稳定性和可观测性。

## 主要优化内容

### 1. 增加默认连接超时时间

**改进前：** 默认超时 3 秒  
**改进后：** 默认超时 10 秒

```rust
// 从 3 秒增加到 10 秒，更好地应对网络延迟
const DEFAULT_CONNECTION_TIMEOUT_SECS: u64 = 10;
```

### 2. 增强日志记录

#### 连接池初始化日志
```
INFO Connection pool for MQTTBrokerPlacementService at 192.168.0.28:1228 initialized (max_open: 100, timeout: 10s)
```

#### 连接获取日志
```
DEBUG Attempting to get connection from MQTTBrokerPlacementService pool at 192.168.0.28:1228 (state: ...)
DEBUG Successfully obtained connection from MQTTBrokerPlacementService pool at 192.168.0.28:1228
```

#### 连接失败日志（带状态对比）
```
WARN MQTTBrokerPlacementService connection pool at 192.168.0.28:1228 has no connection available. 
Error: ..., State before: {...}, State after: {...}
```

#### 连接建立日志（带耗时）
```
INFO Successfully connected to MQTT Broker at 192.168.0.28:1228 (took 1.5s)
WARN Connection to MQTT Broker at 192.168.0.28:1228 took longer than expected: 3.2s
```

### 3. 连接池预热功能

在系统启动时或添加新节点时，可以预热连接池以避免首次请求超时：

```rust
// 单个地址预热
client_pool.warmup_mqtt_broker_pool("192.168.0.28:1228").await?;

// 批量预热
let addrs = vec![
    "192.168.0.28:1228".to_string(),
    "192.168.0.29:1228".to_string(),
];
let results = client_pool.warmup_mqtt_broker_pools(&addrs).await;

for (addr, result) in results {
    match result {
        Ok(_) => println!("✓ {} warmed up successfully", addr),
        Err(e) => println!("✗ {} warmup failed: {}", addr, e),
    }
}
```

### 4. 连接池健康监控

#### 获取单个连接池健康状态
```rust
if let Some(health) = client_pool.get_mqtt_broker_pool_health("192.168.0.28:1228").await {
    println!("Pool Health: {:?}", health);
    println!("Is Healthy: {}", health.is_healthy);
    println!("Active Connections: {}", health.connections);
    println!("Idle Connections: {}", health.idle);
}
```

#### 获取所有连接池健康状态
```rust
let all_health = client_pool.get_all_mqtt_broker_pool_health().await;
for health in all_health {
    if !health.is_healthy {
        eprintln!("⚠️  Unhealthy pool at {}: {:?}", health.addr, health);
    }
}
```

#### 清理不健康的连接池
```rust
if client_pool.clear_mqtt_broker_pool("192.168.0.28:1228") {
    println!("Cleared unhealthy connection pool");
}
```

### 5. 健康状态数据结构

```rust
pub struct PoolHealthStatus {
    pub addr: String,           // 连接地址
    pub max_open: u64,          // 最大连接数
    pub connections: u64,       // 当前连接数
    pub in_use: u64,            // 使用中的连接
    pub idle: u64,              // 空闲连接
    pub is_healthy: bool,       // 是否健康（有可用连接）
}
```

## 使用建议

### 1. 在服务启动时预热连接池

```rust
// 在 BrokerServer 或 MetaService 启动时
pub async fn start(&self) {
    // ... 其他初始化代码 ...
    
    // 预热 MQTT Broker 连接池
    let broker_addrs = self.cache_manager.get_broker_node_addrs();
    let warmup_results = self.client_pool
        .warmup_mqtt_broker_pools(&broker_addrs)
        .await;
    
    for (addr, result) in warmup_results {
        if let Err(e) = result {
            warn!("Failed to warmup connection pool for {}: {}", addr, e);
        }
    }
    
    // ... 启动其他服务 ...
}
```

### 2. 定期监控连接池健康状态

```rust
// 添加定期健康检查任务
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        
        let health_statuses = client_pool.get_all_mqtt_broker_pool_health().await;
        for status in health_statuses {
            if !status.is_healthy {
                warn!(
                    "Unhealthy MQTT Broker pool detected at {}: connections={}, idle={}",
                    status.addr, status.connections, status.idle
                );
                
                // 可选：尝试清理并重建连接池
                client_pool.clear_mqtt_broker_pool(&status.addr);
            }
        }
    }
});
```

### 3. 在添加新节点时预热连接

```rust
// 在 MQTTInnerCallManager 中添加节点时
pub async fn add_node_with_warmup(
    &self,
    cluster: &str,
    node: BrokerNode,
    client_pool: &Arc<ClientPool>,
) -> Result<(), MetaServiceError> {
    // 先预热连接池
    if let Err(e) = client_pool.warmup_mqtt_broker_pool(&node.node_inner_addr).await {
        warn!("Failed to warmup pool for new node {}: {}", node.node_id, e);
    }
    
    // 然后添加节点
    // ... 原有的添加节点逻辑 ...
    
    Ok(())
}
```

## 问题排查指南

### 问题1：连接池仍然超时

**症状：** 日志显示 `connections: 0`

**排查步骤：**
1. 检查网络连通性：`telnet <broker_ip> <port>`
2. 检查 MQTT Broker 服务是否正常运行
3. 查看连接建立耗时，如果超过 10 秒，考虑增加超时时间
4. 检查是否有防火墙规则阻止连接

### 问题2：连接建立慢

**症状：** 日志显示 `took longer than expected: 3.2s`

**可能原因：**
- 网络延迟高
- MQTT Broker 负载高
- DNS 解析慢

**解决方案：**
- 使用 IP 地址而非域名
- 优化网络配置
- 考虑在同一数据中心部署服务

### 问题3：连接池有连接但都在使用中

**症状：** `connections: 10, in_use: 10, idle: 0`

**解决方案：**
- 增加 `max_open_connection` 参数
- 检查是否有连接泄漏（未正确释放）
- 优化请求并发控制

## 监控指标建议

建议监控以下指标：

1. **连接池大小：** `connections`
2. **空闲连接数：** `idle`  
3. **使用中连接数：** `in_use`
4. **连接获取超时次数：** 通过日志 WARN 级别统计
5. **连接建立耗时：** 从日志中提取
6. **连接池健康状态：** `is_healthy`

## 总结

通过这次优化，我们：
- ✅ 将默认超时从 3 秒增加到 10 秒
- ✅ 添加了详细的日志记录，包括状态对比和耗时统计
- ✅ 实现了连接池预热功能，避免首次请求超时
- ✅ 提供了完整的健康监控 API
- ✅ 增强了错误处理和诊断能力

这些改进将显著提高系统的稳定性和可观测性，帮助快速定位和解决连接问题。

