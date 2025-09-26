# MQTT 主题重写

## 什么是 MQTT 主题重写？

很多物联网设备不支持重新配置或升级，修改设备业务主题会非常困难。主题重写功能可以帮助使这种业务升级变得更容易：通过给 RobustMQ 设置一套规则，它可以在订阅、发布时改变将原有主题重写为新的目标主题。

主题重写是 RobustMQ 支持的 MQTT 扩展功能，允许在消息发布和订阅时动态修改主题名称，实现主题的透明转换和业务逻辑的灵活调整。

## 何时使用 MQTT 主题重写？

主题重写适用于以下场景：

- **设备兼容性**：旧设备使用固定主题，需要与新系统集成
- **业务升级**：在不修改客户端代码的情况下调整主题结构
- **系统迁移**：将消息从旧主题结构迁移到新主题结构
- **多租户支持**：为不同租户提供个性化的主题命名空间
- **协议适配**：适配不同厂商设备的主题命名规范
- **安全隔离**：通过主题重写实现消息路由和访问控制

## 主题重写的特性

- **动态重写**：在运行时动态修改主题名称
- **规则匹配**：支持通配符和正则表达式匹配
- **多规则支持**：可以配置多个重写规则
- **方向控制**：可以分别控制发布和订阅的重写
- **变量替换**：支持使用客户端信息进行主题构建
- **兼容性**：与保留消息和延迟发布功能兼容

## 主题重写规则配置

主题重写规则由以下部分组成：

- **action**：规则作用范围（`publish`、`subscribe`、`all`）
- **source_topic**：源主题过滤器（支持通配符）
- **dest_topic**：目标主题模板
- **re**：正则表达式（用于提取主题信息）

### 规则类型

- **publish 规则**：仅匹配 PUBLISH 报文携带的主题
- **subscribe 规则**：仅匹配 SUBSCRIBE、UNSUBSCRIBE 报文携带的主题
- **all 规则**：对 PUBLISH、SUBSCRIBE 和 UNSUBSCRIBE 报文都生效

### 变量支持

目标表达式支持以下变量：

- `$N`：正则表达式提取的第 N 个元素
- `${clientid}`：客户端 ID
- `${username}`：客户端用户名

## 通过 MQTTX 使用主题重写

### 使用 MQTTX CLI

1. **配置主题重写规则**

   假设已配置以下重写规则：

   ```text
   规则1: y/+/z/# → y/z/$2 (正则: ^y/(.+)/z/(.+)$)
   规则2: x/# → z/y/x/$1 (正则: ^x/y/(.+)$)
   规则3: x/y/+ → z/y/$1 (正则: ^x/y/(\d+)$)
   ```

2. **测试主题重写**

   ```bash
   # 订阅会被重写的主题
   mqttx sub -t 'y/a/z/b' -h '117.72.92.117' -p 1883 -v
   # 实际订阅: y/z/b
   
   mqttx sub -t 'x/y/123' -h '117.72.92.117' -p 1883 -v
   # 实际订阅: z/y/123
   
   mqttx sub -t 'x/1/2' -h '117.72.92.117' -p 1883 -v
   # 实际订阅: x/1/2 (不匹配正则表达式)
   ```

3. **发布到重写后的主题**

   ```bash
   # 发布到原始主题，会被重写
   mqttx pub -t 'y/device1/z/data' -m '{"temperature":25.5}' -h '117.72.92.117' -p 1883
   # 实际发布到: y/z/data
   
   mqttx pub -t 'x/y/sensor' -m '{"humidity":60}' -h '117.72.92.117' -p 1883
   # 实际发布到: z/y/x/sensor
   ```

### 实际应用示例

#### 设备主题标准化

```bash
# 旧设备使用非标准主题
mqttx pub -t 'device/001/temp' -m '{"value":25.5}' -h '117.72.92.117' -p 1883

# 配置重写规则: device/+/temp → sensors/$1/temperature
# 实际发布到: sensors/001/temperature

# 新系统订阅标准化主题
mqttx sub -t 'sensors/+/temperature' -h '117.72.92.117' -p 1883 -v
```

#### 多租户主题隔离

```bash
# 租户A的设备发布消息
mqttx pub -t 'tenantA/sensor/data' -m '{"value":30}' -h '117.72.92.117' -p 1883

# 配置重写规则: tenantA/# → ${username}/tenantA/#
# 实际发布到: user1/tenantA/sensor/data

# 租户A订阅自己的主题
mqttx sub -t 'user1/tenantA/#' -h '117.72.92.117' -p 1883 -v
```

#### 协议版本升级

```bash
# 旧版本设备使用v1主题
mqttx pub -t 'v1/sensor/temperature' -m '{"temp":25}' -h '117.72.92.117' -p 1883

# 配置重写规则: v1/# → v2/legacy/#
# 实际发布到: v2/legacy/sensor/temperature

# 新版本系统订阅v2主题
mqttx sub -t 'v2/legacy/#' -h '117.72.92.117' -p 1883 -v
```

#### 地理位置主题路由

```bash
# 设备按地理位置发布
mqttx pub -t 'beijing/sensor/air' -m '{"pm25":50}' -h '117.72.92.117' -p 1883

# 配置重写规则: beijing/# → region/north/beijing/#
# 实际发布到: region/north/beijing/sensor/air

# 区域监控系统订阅
mqttx sub -t 'region/north/#' -h '117.72.92.117' -p 1883 -v
```

#### 设备类型分类

```bash
# 不同类型设备发布消息
mqttx pub -t 'device/001/status' -m '{"online":true}' -h '117.72.92.117' -p 1883
mqttx pub -t 'device/002/status' -m '{"online":true}' -h '117.72.92.117' -p 1883

# 配置重写规则: device/+/status → devices/${clientid}/status
# 实际发布到: devices/device001/status, devices/device002/status

# 设备管理系统订阅
mqttx sub -t 'devices/+/status' -h '117.72.92.117' -p 1883 -v
```

## 主题重写与共享订阅

主题重写在作用于客户端的订阅/取消订阅共享订阅主题时，仅对实际主题生效。即只对共享订阅主题去除前缀 `$share/<group-name>/` 或 `$queue` 之后的部分生效。

例如：

- 客户端订阅 `$share/group/t/1` 时，仅尝试匹配并重写 `t/1`
- 客户端订阅 `$queue/t/2` 时，仅尝试匹配并重写 `t/2`

```bash
# 共享订阅主题重写示例
mqttx sub -t '$share/group1/y/a/z/b' -h '117.72.92.117' -p 1883 -v
# 实际订阅: $share/group1/y/z/b (重写规则作用于 y/a/z/b)

mqttx sub -t '$queue/x/y/123' -h '117.72.92.117' -p 1883 -v
# 实际订阅: $queue/z/y/123 (重写规则作用于 x/y/123)
```

## 主题重写规则优先级

当多个规则匹配同一个主题时，RobustMQ 按照以下优先级处理：

1. **配置顺序**：按照配置文件中规则的顺序执行
2. **第一个匹配**：使用第一个匹配的规则进行重写
3. **正则匹配**：如果正则表达式不匹配，重写失败，不会尝试其他规则

## 与保留消息和延迟发布的结合

主题重写可以与保留消息和延迟发布功能结合使用：

```bash
# 延迟发布结合主题重写
mqttx pub -t '$delayed/10/y/device/data' -m '{"delayed":true}' -h '117.72.92.117' -p 1883
# 实际发布到: $delayed/10/y/z/data (重写规则作用于 y/device/data)

# 保留消息结合主题重写
mqttx pub -t 'x/y/config' -m '{"retain":true}' --retain -h '117.72.92.117' -p 1883
# 实际发布到: z/y/x/config (重写规则作用于 x/y/config)
```

## 注意事项

1. **ACL 检查**：发布/订阅授权检查在主题重写之前执行
2. **规则顺序**：规则按配置顺序执行，第一个匹配的规则生效
3. **正则匹配**：正则表达式不匹配时，重写失败
4. **共享订阅**：共享订阅主题重写仅对实际主题部分生效
5. **性能影响**：每个主题都需要匹配所有规则，影响性能
6. **兼容性**：确保重写后的主题与现有系统兼容
