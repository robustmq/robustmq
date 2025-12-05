# get_local_ip() 智能IP检测优化

## 概述

优化了 `get_local_ip()` 函数，使其能够智能选择最适合的网络IP地址，避免选择临时网络（如Docker、VPN）的IP。

## 改进内容

### 1. 三层优先级策略

```rust
// 优先级 1: 环境变量
export ROBUSTMQ_BROKER_IP=192.168.1.100

// 优先级 2: 智能检测
// 自动选择最佳网络接口

// 优先级 3: 回退
// 127.0.0.1
```

### 2. 智能检测规则

#### 排除的接口：
- ❌ 回环地址 (`lo0`, `127.0.0.1`)
- ❌ Docker 网络 (`docker0`, `br-*`)
- ❌ VPN/隧道 (`utun*`, `tun*`, `tap*`, `ppp*`)
- ❌ Apple Wireless Direct Link (`awdl*`, `llw*`)
- ❌ 链路本地地址 (`169.254.x.x`)
- ❌ IPv6 地址

#### 选择优先级：
1. **物理网卡的私有IP** (最高优先级)
   - macOS: `en0`, `en1`
   - Linux: `eth0`, `ens33`, `enp0s3`
   - WiFi: `wlan0`

2. **私有网段优先级**
   - `192.168.x.x` > `10.x.x.x` > `172.16-31.x.x`

3. **公网IP** (最低优先级)

## 使用方式

### 方式 1: 自动检测（推荐开发环境）

```rust
use common_base::tools::get_local_ip;

fn main() {
    let ip = get_local_ip();
    println!("Detected IP: {}", ip);
}
```

**输出示例：**
```
INFO Auto-detected local IP: 192.168.100.100
INFO Selected network interface: en0 -> 192.168.100.100
```

### 方式 2: 环境变量（推荐容器/CI环境）

```bash
# 设置环境变量
export ROBUSTMQ_BROKER_IP=10.0.0.1

# 启动服务
./target/debug/broker-server
```

**日志输出：**
```
INFO Using IP from environment variable ROBUSTMQ_BROKER_IP: 10.0.0.1
```

### 方式 3: 配置文件（推荐生产环境）

在 `config/server.toml` 中添加：

```toml
broker_ip = "192.168.1.100"
```

然后在代码中：

```rust
use common_config::broker::broker_config;

let config = broker_config();
let ip = config.broker_ip.clone().unwrap_or_else(get_local_ip);
```

## 实际场景示例

### 场景 1: 开发环境（macOS）

**网络接口：**
```
en0: 192.168.100.100     <- WiFi/以太网
utun4: 10.255.208.17     <- VPN
docker0: 172.17.0.1      <- Docker
```

**检测结果：** `192.168.100.100` ✅

### 场景 2: Linux 服务器

**网络接口：**
```
eth0: 10.0.1.100         <- 内网
docker0: 172.17.0.1      <- Docker
```

**检测结果：** `10.0.1.100` ✅

### 场景 3: 容器环境

```dockerfile
ENV ROBUSTMQ_BROKER_IP=0.0.0.0
```

**检测结果：** `0.0.0.0` (监听所有接口) ✅

## 测试

运行测试验证功能：

```bash
cargo test --package common-base --lib tools::tests
```

**测试覆盖：**
- ✅ 环境变量优先级
- ✅ 物理网卡识别
- ✅ 私有IP判断
- ✅ 接口过滤规则

## 排查问题

### 如何查看选择的IP？

启动服务时查看日志：

```bash
./target/debug/broker-server 2>&1 | grep "Selected\|Auto-detected\|Using IP"
```

### 强制使用特定IP

```bash
# 临时设置
ROBUSTMQ_BROKER_IP=127.0.0.1 ./target/debug/broker-server

# 或修改配置
echo 'broker_ip = "127.0.0.1"' >> config/server.toml
```

### 列出所有网络接口

```bash
# macOS
ifconfig -a | grep -E "^[a-z]|inet "

# Linux
ip addr show
```

## 技术细节

### 代码位置

- **实现文件**: `src/common/base/src/tools.rs`
- **主函数**: `get_local_ip()`
- **辅助函数**:
  - `get_local_ip_smart()` - 智能检测逻辑
  - `should_skip_interface()` - 接口过滤
  - `is_physical_interface()` - 物理网卡判断
  - `is_private_ip()` - 私有IP判断
  - `select_best_ip()` - 最优IP选择

### 依赖库

```toml
[dependencies]
local-ip-address = "0.6.1"
```

## 向后兼容

✅ 完全向后兼容，不影响现有代码

- 如果没有设置环境变量，行为与之前类似（但更智能）
- 配置文件优先级保持不变
- 回退逻辑保持 `127.0.0.1`

## 后续改进建议

1. **配置文件支持排除列表**
   ```toml
   [network]
   excluded_interfaces = ["docker.*", "br-.*"]
   preferred_interfaces = ["eth0", "en0"]
   ```

2. **支持指定网卡名称**
   ```toml
   network_interface = "eth0"
   ```

3. **健康检查**
   - 定期验证选择的IP是否仍然可用
   - 在心跳中携带IP变更信息

## 相关链接

- Issue: #xxx
- PR: #xxx
- 相关优化: `broker-core/src/cluster.rs` 节点注册逻辑
