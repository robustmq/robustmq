# 指标

# 字节总数指标

| Metric Name      | Description |
|------------------|-------------|
| `bytes_received` | 已接受字节数      |
| `bytes_sent`     | 已发送字节数      |

# 报文指标

| Metric Name                    | Description                                                  |
|--------------------------------|--------------------------------------------------------------|
| `packets_received`             | 已接受报文数                                                       |
| `packets_connect_received`     | 接收的CONNECT报文数量                                               |
| `packets_publish_received`     | 接收的PUBLISH报文数量                                               |
| `packets_puback_received`      | 接收的PUBACK报文数量                                                |
| `packets_pubrec_received`      | 接收的PUBREC报文数量                                                |
| `packets_pubrel_received`      | 接收的PUBREL报文数量                                                |
| `packets_pubcomp_received`     | 接收的PUBCOMP报文数量                                               |
| `packets_subscribe_received`   | 接收的SUBSCRIBE报文数量                                             |
| `packets_unsubscribe_received` | 接收的UNSUBSCRIBE报文数量                                           |
| `packets_pingreq_received`     | 接收的PINGREQ报文数量                                               |
| `packets_disconnect_received`  | 接收的DISCONNECT报文数量                                            |
| `packets_auth_received`        | 接收的AUTH报文数量                                                  |
| `packets_sent`                 | 已发送报文数                                                       |
| `packets_connack_sent`         | 发送的CONNACK报文数量                                               |
| `packets_publish_sent`         | 发送的PUBLISH报文数量                                               |
| `packets_puback_sent`          | 发送的PUBACK报文数量                                                |
| `packets_pubrec_sent`          | 发送的PUBREC报文数量                                                |
| `packets_pubrel_sent`          | 发送的PUBREL报文数量                                                |
| `packets_pubcomp_sent`         | 发送的PUBCOMP报文数量                                               |
| `packets_suback_sent`          | 发送的SUBACK报文数量                                                |
| `packets_unsuback_sent`        | 发送的UNSUBACK报文数量                                              |
| `packets_pingresp_sent`        | 发送的PINGRESP报文数量                                              |
| `packets_disconnect_sent`      | 发送的DISCONNECT报文数量                                            |
| `packets_auth_sent`            | 发送的AUTH报文数量                                                  |
| `packets_connack_auth_error`   | 发送的原因码为0x86和0x87的CONNACK报文数量                                 |
| `packets_connack_error`        | 发送的不为0x00的CONNACK报文数量，此指标的值大于等于`packects_connack_auth_error` |


# 消息指标


| Metric Name          | Description    |
|----------------------|----------------|
| `mmessages_delayed`  | 存储的延迟发布的消息数量   |
| `messages_delivered` | 内部转发到订阅进程的消息数量 |

# 事件指标

| Metric Name      | Description               |
|------------------|---------------------------|
| `client_connack` | 客户端收到连接确认(CONNACK)消息的次数   |
| `client_connect` | 客户端发起连接请求的次数，包括成功和失败的连接请求 |


# 会话指标

| Metric Name          | Description |
|----------------------|-------------|
| `sessions_created`   | 创建的会话数量     |
| `sessions_discarded` | 已被丢弃的会话数量   |

# 认证和授权指标

| Metric Name           | Description                  |
|-----------------------|------------------------------|
| `authorization_allow` | 授权总的通过次数（包括命中缓存和规则未匹配时默认通过的  |
| `authorization_deny`  | 授权总的拒绝次数（包括命中缓存和规则未匹配时默认拒绝的） |

# 消息分发指标

| Metric Name                   | Description                |
|-------------------------------|----------------------------|
| `delivery_dropped_too_large`  | 发送时由于长度超过限制而丢弃的消息数量        |
| `delivery_dropped_queue_full` | 发送时由于消息队列满而被丢弃的QoS不为0的消息数量 |


# 统计指标

| Metric Name         | Description |
|---------------------|-------------|
| `connections_count` | 当前连接数量      |
| `connections_max`   | 连接数量的历史最大值  |
