# 规则引擎总览

RobustMQ 规则引擎用于在 Connector 内做轻量数据处理。  
当前目标不是替代 Flink 这类流处理系统，而是覆盖 MQTT 场景里常见的简单加工任务，减少外部系统依赖。

## 源数据格式

| 源数据格式 | 简要说明 | Demo（短） | 是否支持 | 优先级 |
| --- | --- | --- | --- | --- |
| JSON Object | 单条结构化对象 | `{"device":"d1","temp":36.5}` | ✅ | 高 |
| JSON Array | 多条对象数组 | `[{"device":"d1"},{"device":"d2"}]` | ✅ | 高 |
| JSON Lines | 每行一个 JSON 对象 | `{"d":"d1"}\n{"d":"d2"}` | ✅ | 中 |
| key=value 行日志 | 键值对文本，按分隔符解析 | `ts=2026-03-03 level=INFO temp=36.5` | ✅ | 高 |
| 无 key 行日志 | 固定列顺序文本 | `2026-03-03 INFO d1 36.5` | ✅ | 高 |
| CSV | 逗号分隔表格文本 | `ts,level,device,temp` | ✅ | 高 |
| 纯文本 | 非结构化或半结构化字符串 | `device d1 temp 36.5` | ✅ | 中 |
| bytes（二进制） | 原始二进制负载 | `0x89 0x50 0x4E 0x47 ...` | ✅ | 低 |
| Protobuf | 二进制协议，需要 schema | `Message<DeviceData>(binary)` | ❌ | 低 |
| XML | 标签化结构文本 | `<msg><device>d1</device></msg>` | ❌ | 中 |

## 算子列表

### 过滤与条件

| 算子 | 作用 | 典型场景 | 完成 |
| --- | --- | --- | --- |
| `filter` | CEL 表达式条件过滤，不满足则丢弃消息 | 仅保留温度 > 0 的数据，过滤心跳包 | ❌ |
| `case_when` | 多分支条件赋值，根据条件输出不同字段值 | `code=1→status="ok"`, `code=2→status="warn"` | ❌ |
| `coalesce` | 返回第一个非空字段值，可指定默认值 | 字段可能缺失时提供兜底值 | ❌ |

### 字段操作

| 算子 | 作用 | 典型场景 | 完成 |
| --- | --- | --- | --- |
| `extract` | 按字段映射提取字段，支持点路径和 JSON Pointer，不存在字段填 `"-"` | 从复杂 JSON 投影出 `topic / ip / alarm_active` 等关键字段 | ✅ |
| `set` | 新增字段或覆盖字段，值支持 CEL 表达式 | 写入 `tenant`、计算 `temp_f = temp * 1.8 + 32` | ❌ |
| `delete` | 删除一个或多个顶层字段（按 key 列表） | 删除中间态字段、敏感字段、调试字段 | ✅ |
| `rename` | 顶层字段重命名（`old_key -> new_key`） | 把异构设备字段名统一映射 | ✅ |
| `keep_only` | 只保留指定顶层字段（按 key 列表），其余全部删除 | 脱敏后只保留必要字段再写存储 | ✅ |
| `flatten` | 嵌套 JSON 展平为顶层字段，支持自定义前缀 | `{"a":{"b":1}}` → `{"a.b":1}` | ❌ |
| `nest` | 将多个字段打包为嵌套结构 | 把 `lat / lon` 合并为 `location: {lat, lon}` | ❌ |
| `merge` | 合并两个 Map 对象，key 冲突时可配置覆盖策略 | 用元数据扩充设备上报字段 | ❌ |
| `expand` | 把数组字段拆分为多条独立消息（1→N） | `payload.readings: [...]` 每条拆成单独消息 | ❌ |

### 类型转换与判断

| 算子 | 作用 | 典型场景 | 完成 |
| --- | --- | --- | --- |
| `cast` | 字段类型转换（`to_int / to_float / to_string / to_bool`） | 字符串 `"36.5"` 转 float，`"true"` 转 bool | ❌ |
| `is_null / is_not_null` | 判断字段是否为空 | filter 里的防御性校验 | ❌ |
| `is_int / is_float / is_bool / is_str` | 判断字段类型，返回 bool | 动态类型数据校验 | ❌ |
| `is_array / is_map` | 判断字段是否为数组或 Map，返回 bool | 上报格式校验 | ❌ |

### 字符串

| 算子 | 作用 | 典型场景 | 完成 |
| --- | --- | --- | --- |
| `trim / upper / lower` | 去首尾空白、大小写转换 | 设备 ID 标准化 | ❌ |
| `split / concat / replace` | 字符串分割、拼接、替换 | topic 路径拆分、字段值拼接 | ❌ |
| `substr / strlen` | 子串截取、字符串长度 | 截断超长字段值 | ❌ |
| `regex_match` | 正则匹配，返回 bool | filter 里判断字段值格式 | ❌ |
| `regex_extract` | 正则提取捕获组，返回数组 | 从日志字符串里提取 IP、时间戳 | ❌ |
| `regex_replace` | 正则替换 | 脱敏手机号、清洗格式 | ❌ |
| `template_render` | 字符串模板渲染，支持 `${field}` 语法 | 拼接 `"device-${device_id}-${region}"` | ❌ |
| `url_encode / url_decode` | URL 编解码 | 处理含特殊字符的字段值 | ❌ |

### 数学运算

| 算子 | 作用 | 典型场景 | 完成 |
| --- | --- | --- | --- |
| `abs / ceil / floor / round` | 绝对值、取整 | 传感器值取整 | ❌ |
| `sqrt / power / log / log10` | 幂次与对数 | 信号强度 dB 换算 | ❌ |
| `clamp(x, min, max)` | 区间截断，超出范围取边界值 | 异常值截断防止下游越界 | ❌ |
| `normalize(field, min, max)` | 归一化到 [0,1]，或自定义目标区间 | AI 训练数据预处理、异常分数标准化 | ❌ |
| `unit_convert(field, from, to)` | 工程单位换算，内置单位库（℃/℉、Pa/bar、mV/V 等） | 传感器单位标准化，无需外部转换代码 | ❌ |
| `random()` | 返回 [0,1) 随机数 | 消息采样（只处理 10% 数据） | ❌ |

### 时间

| 算子 | 作用 | 典型场景 | 完成 |
| --- | --- | --- | --- |
| `now_timestamp` | 返回当前 Unix 时间戳（秒/毫秒） | 给消息注入处理时间 | ❌ |
| `now_rfc3339` | 返回当前时间的 RFC3339 字符串 | 日志类消息的时间标注 | ❌ |
| `unix_ts_to_rfc3339` | Unix 时间戳转 RFC3339 字符串 | 设备时间戳格式化后写存储 | ❌ |
| `rfc3339_to_unix_ts` | RFC3339 字符串转 Unix 时间戳 | 统一时间字段为数值方便计算 | ❌ |
| `format_date` | 按格式字符串格式化日期时间 | 输出 `"2026-03-03 15:00:00"` 格式 | ❌ |
| `date_diff` | 计算两个时间点的差值 | 设备上报延迟、超时检测 | ❌ |
| `timezone_convert` | 时区转换 | 全球设备统一转 UTC | ❌ |

### JSON 操作

| 算子 | 作用 | 典型场景 | 完成 |
| --- | --- | --- | --- |
| `json_decode / json_encode` | JSON 字符串序列化/反序列化 | payload 是 JSON 字符串时先 decode | ❌ |
| `map_get / map_put` | Map 字段读写，支持嵌套路径 | 安全读取可能不存在的嵌套字段 | ❌ |
| `map_keys / map_values` | 返回 Map 的所有 key 或 value 列表 | 动态枚举字段 | ❌ |
| `map_to_entries` | Map 转为 `[{key, value}]` 数组 | 把 map 转成可迭代结构 | ❌ |
| `json_path` | JSONPath 语法查询，如 `$.store.book[*].author` | 复杂嵌套 JSON 字段提取 | ❌ |
| `jq` | jq 语法完整支持，复杂 JSON 变换 | 一行 jq 替代多步字段操作 | ❌ |

### 数组操作

| 算子 | 作用 | 典型场景 | 完成 |
| --- | --- | --- | --- |
| `map_array` | 对数组每个元素应用转换函数 | 批量给数组里每条记录加字段 | ❌ |
| `filter_array` | 对数组按条件过滤元素 | 过滤 readings 数组中的无效值 | ❌ |
| `first / last / nth` | 取数组首尾或第 N 个元素 | 取最新一条读数 | ❌ |
| `contains` | 判断数组是否包含指定元素 | 过滤设备类型在白名单内的消息 | ❌ |
| `length` | 返回数组长度 | 判断批量上报条数 | ❌ |

### 编解码

| 算子 | 作用 | 典型场景 | 完成 |
| --- | --- | --- | --- |
| `decode` | 将输入 `Bytes` 解析为统一记录格式 `Vec<Map<String, Value>>`，支持 JSON/行日志/CSV/纯文本/bytes | 统一不同来源数据形态，供后续算子处理 | ✅ |
| `encode` | 将处理后的统一记录格式序列化回 `Bytes`，支持 JSON/行日志/CSV/纯文本/bytes | 按目标 Connector 所需格式输出数据 | ✅ |
| `base64_encode / base64_decode` | Base64 编解码 | 二进制内容转文本传输 | ❌ |
| `bin2hexstr / hexstr2bin` | 十六进制字符串与字节互转 | 调试工业设备原始帧 | ❌ |
| `gzip / gunzip` | 压缩/解压 | 压缩前写存储，或解压设备上报数据 | ❌ |
| `csv_decode / csv_encode` | CSV 字符串与结构化数据互转 | 批量设备数据表格格式 | ❌ |
| `bytes_decode(field, format)` | 按格式解析原始字节流，支持 `int16_be / float32_le` 等工业协议格式 | Modbus/PLC 原始帧解析 | ❌ |
| `protobuf_decode / protobuf_encode` | Protobuf 编解码，需提供 schema | 工业设备 protobuf 上报 | ❌ |
| `avro_decode / avro_encode` | Avro 编解码 | Kafka 生态数据管道 | ❌ |
| `msgpack_decode / msgpack_encode` | MessagePack 编解码 | 低功耗设备压缩上报 | ❌ |

### 哈希与安全

| 算子 | 作用 | 典型场景 | 完成 |
| --- | --- | --- | --- |
| `md5 / sha256 / sha512` | 哈希计算 | 内容摘要、去重标识 | ❌ |
| `hmac_sha256` | HMAC-SHA256 签名 | 消息完整性校验 | ❌ |
| `aes_encrypt / aes_decrypt` | AES 对称加密/解密 | 字段级敏感数据加密 | ❌ |
| `mask_field(field, strategy)` | 字段脱敏（置空/置零/截断/哈希替换） | 手机号、坐标等隐私字段处理 | ❌ |
| `uuid4` | 生成 UUID v4 | 给每条消息注入唯一 trace ID | ❌ |

### 边缘专属

| 算子 | 作用 | 典型场景 | 完成 |
| --- | --- | --- | --- |
| `threshold_alert(field, min, max, label)` | 字段值超阈值时自动注入告警标签 | 温度超限自动打 `alert=overheat` | ❌ |
| `moving_avg(field, n)` | N 点移动平均（有限本地状态，重启丢失） | 边缘侧简单降噪 | ❌ |
| `deduplicate(field, ttl_ms)` | 按字段值去重，TTL 滑动窗口（有限本地状态） | 过滤设备重复上报 | ❌ |

## 执行模型（基础版）

| 项目 | 说明 |
| --- | --- |
| 规则表达 | 规则以 JSON 数组表示 |
| 执行顺序 | 数组有序，按顺序执行 |
| 数据流 | 上一个算子的输出作为下一个算子的输入 |
| 异常处理 | 任一环节失败按规则错误策略处理（如跳过或终止） |

## 当前边界

基础版只聚焦“轻量、可预测、可验证”的链式处理：

| 能力项 | 基础版状态 |
| --- | --- |
| 跨消息聚合 | 不支持 |
| 窗口计算 | 不支持 |
| 状态存储与回溯计算 | 不支持 |

后续会在保持链路稳定的前提下，逐步扩展更多算子能力。
