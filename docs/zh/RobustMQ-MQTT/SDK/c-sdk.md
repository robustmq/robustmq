# 使用 C SDK 连接 RobustMQ

## 概述

Eclipse Paho C 是一个功能齐全的 MQTT C 语言客户端库，使用 ANSI C 编写，可用于连接 RobustMQ MQTT Broker。该客户端库提供同步和异步两种 API，适用于不同的应用场景。

## 客户端库介绍

### Eclipse Paho C vs Eclipse Paho Embedded C

- **Eclipse Paho C**：功能完整的 MQTT 客户端，适用于桌面和服务器环境
- **Eclipse Paho Embedded C**：轻量级版本，主要针对 mbed、Arduino 和 FreeRTOS 等嵌入式环境

### API 类型

该客户端提供两种 API 类型：

- **同步 API** (MQTTClient)：
  - 更简单易用，某些调用会阻塞直到操作完成
  - 适合主线程环境，编程更容易

- **异步 API** (MQTTAsync)：
  - 只有一个阻塞调用 `waitForCompletion`
  - 通过回调函数进行结果通知
  - 更适用于非主线程环境

## 安装依赖

### Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install libpaho-mqtt-dev
```

### CentOS/RHEL

```bash
sudo yum install epel-release
sudo yum install paho-c-devel
```

### 从源码编译

```bash
git clone https://github.com/eclipse/paho.mqtt.c.git
cd paho.mqtt.c
mkdir build
cd build
cmake ..
make
sudo make install
```

## 连接示例

### 基础连接和发布示例

```c
#include "stdio.h"
#include "stdlib.h"
#include "string.h"
#include "MQTTClient.h"

#define ADDRESS     "tcp://localhost:1883"
#define CLIENTID    "robustmq_c_client"
#define TOPIC       "test/topic"
#define PAYLOAD     "Hello RobustMQ!"
#define QOS         1
#define TIMEOUT     10000L

int main(int argc, char* argv[])
{
    MQTTClient client;
    MQTTClient_connectOptions conn_opts = MQTTClient_connectOptions_initializer;
    MQTTClient_message pubmsg = MQTTClient_message_initializer;
    MQTTClient_deliveryToken token;
    int rc;

    // 创建 MQTT 客户端
    MQTTClient_create(&client, ADDRESS, CLIENTID,
        MQTTCLIENT_PERSISTENCE_NONE, NULL);

    // 设置连接参数
    conn_opts.keepAliveInterval = 20;
    conn_opts.cleansession = 1;

    // 连接到 RobustMQ Broker
    if ((rc = MQTTClient_connect(client, &conn_opts)) != MQTTCLIENT_SUCCESS)
    {
        printf("Failed to connect to RobustMQ, return code %d\n", rc);
        exit(-1);
    }

    printf("Connected to RobustMQ successfully\n");

    // 发布消息
    pubmsg.payload = PAYLOAD;
    pubmsg.payloadlen = strlen(PAYLOAD);
    pubmsg.qos = QOS;
    pubmsg.retained = 0;

    MQTTClient_publishMessage(client, TOPIC, &pubmsg, &token);
    printf("Publishing message: %s\n", PAYLOAD);
    printf("On topic: %s\n", TOPIC);

    rc = MQTTClient_waitForCompletion(client, token, TIMEOUT);
    printf("Message with delivery token %d delivered\n", token);

    // 断开连接
    MQTTClient_disconnect(client, 10000);
    MQTTClient_destroy(&client);

    printf("Disconnected from RobustMQ\n");
    return rc;
}
```

### 订阅消息示例

```c
#include "stdio.h"
#include "stdlib.h"
#include "string.h"
#include "MQTTClient.h"

#define ADDRESS     "tcp://localhost:1883"
#define CLIENTID    "robustmq_c_subscriber"
#define TOPIC       "test/topic"
#define QOS         1
#define TIMEOUT     10000L

// 消息到达回调函数
int messageArrived(void *context, char *topicName, int topicLen, MQTTClient_message *message)
{
    printf("Message arrived:\n");
    printf("Topic: %s\n", topicName);
    printf("Message: %.*s\n", message->payloadlen, (char*)message->payload);
    printf("QoS: %d\n", message->qos);

    MQTTClient_freeMessage(&message);
    MQTTClient_free(topicName);
    return 1;
}

// 连接丢失回调函数
void connectionLost(void *context, char *cause)
{
    printf("Connection lost: %s\n", cause);
}

int main(int argc, char* argv[])
{
    MQTTClient client;
    MQTTClient_connectOptions conn_opts = MQTTClient_connectOptions_initializer;
    int rc;

    // 创建 MQTT 客户端
    MQTTClient_create(&client, ADDRESS, CLIENTID,
        MQTTCLIENT_PERSISTENCE_NONE, NULL);

    // 设置回调函数
    MQTTClient_setCallbacks(client, NULL, connectionLost, messageArrived, NULL);

    // 设置连接参数
    conn_opts.keepAliveInterval = 20;
    conn_opts.cleansession = 1;

    // 连接到 RobustMQ Broker
    if ((rc = MQTTClient_connect(client, &conn_opts)) != MQTTCLIENT_SUCCESS)
    {
        printf("Failed to connect to RobustMQ, return code %d\n", rc);
        exit(-1);
    }

    printf("Connected to RobustMQ successfully\n");

    // 订阅主题
    printf("Subscribing to topic: %s\n", TOPIC);
    MQTTClient_subscribe(client, TOPIC, QOS);

    // 等待消息
    printf("Waiting for messages...\n");
    printf("Press Q<Enter> to quit\n\n");

    int ch;
    do {
        ch = getchar();
    } while (ch != 'Q' && ch != 'q');

    // 取消订阅并断开连接
    MQTTClient_unsubscribe(client, TOPIC);
    MQTTClient_disconnect(client, 10000);
    MQTTClient_destroy(&client);

    printf("Disconnected from RobustMQ\n");
    return rc;
}
```

## 高级功能

### SSL/TLS 连接

```c
#include "MQTTClient.h"

#define ADDRESS     "ssl://localhost:1885"
#define CLIENTID    "robustmq_c_ssl_client"

int main()
{
    MQTTClient client;
    MQTTClient_connectOptions conn_opts = MQTTClient_connectOptions_initializer;
    MQTTClient_SSLOptions ssl_opts = MQTTClient_SSLOptions_initializer;
    int rc;

    MQTTClient_create(&client, ADDRESS, CLIENTID,
        MQTTCLIENT_PERSISTENCE_NONE, NULL);

    // 配置 SSL 选项
    ssl_opts.enableServerCertAuth = 1;
    ssl_opts.trustStore = "/path/to/ca.crt";  // CA 证书路径
    ssl_opts.keyStore = "/path/to/client.crt"; // 客户端证书路径
    ssl_opts.privateKey = "/path/to/client.key"; // 客户端私钥路径

    // 设置连接参数
    conn_opts.keepAliveInterval = 20;
    conn_opts.cleansession = 1;
    conn_opts.ssl = &ssl_opts;

    // 连接到 RobustMQ Broker (SSL)
    if ((rc = MQTTClient_connect(client, &conn_opts)) != MQTTCLIENT_SUCCESS)
    {
        printf("Failed to connect to RobustMQ with SSL, return code %d\n", rc);
        exit(-1);
    }

    printf("Connected to RobustMQ with SSL successfully\n");

    // ... 其他操作 ...

    MQTTClient_disconnect(client, 10000);
    MQTTClient_destroy(&client);
    return rc;
}
```

### 用户名密码认证

```c
#include "MQTTClient.h"

int main()
{
    MQTTClient client;
    MQTTClient_connectOptions conn_opts = MQTTClient_connectOptions_initializer;
    int rc;

    MQTTClient_create(&client, "tcp://localhost:1883", "robustmq_auth_client",
        MQTTCLIENT_PERSISTENCE_NONE, NULL);

    // 设置认证信息
    conn_opts.keepAliveInterval = 20;
    conn_opts.cleansession = 1;
    conn_opts.username = "your_username";
    conn_opts.password = "your_password";

    // 连接到 RobustMQ Broker
    if ((rc = MQTTClient_connect(client, &conn_opts)) != MQTTCLIENT_SUCCESS)
    {
        printf("Authentication failed, return code %d\n", rc);
        exit(-1);
    }

    printf("Authenticated and connected to RobustMQ successfully\n");

    // ... 其他操作 ...

    MQTTClient_disconnect(client, 10000);
    MQTTClient_destroy(&client);
    return rc;
}
```

## 编译和运行

### 编译命令

```bash
# 基础编译
gcc -o mqtt_client mqtt_client.c -lpaho-mqtt3c

# 如果使用 SSL
gcc -o mqtt_ssl_client mqtt_ssl_client.c -lpaho-mqtt3cs

# 如果使用异步 API
gcc -o mqtt_async_client mqtt_async_client.c -lpaho-mqtt3a
```

### 运行示例

```bash
# 运行发布客户端
./mqtt_client

# 运行订阅客户端
./mqtt_subscriber
```

## 连接参数说明

### 基础连接参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `ADDRESS` | RobustMQ Broker 地址 | tcp://localhost:1883 |
| `CLIENTID` | 客户端唯一标识 | 自动生成 |
| `keepAliveInterval` | 心跳间隔（秒） | 60 |
| `cleansession` | 是否清除会话 | 1 |

### RobustMQ 支持的协议端口

| 协议 | 端口 | 说明 |
|------|------|------|
| MQTT | 1883 | 标准 MQTT 端口 |
| MQTT over SSL | 1885 | 加密 MQTT 连接 |
| MQTT over WebSocket | 8083 | WebSocket 连接 |
| MQTT over WSS | 8085 | 加密 WebSocket 连接 |

## 最佳实践

### 1. 错误处理

```c
// 连接错误处理
if ((rc = MQTTClient_connect(client, &conn_opts)) != MQTTCLIENT_SUCCESS)
{
    switch(rc) {
        case MQTTCLIENT_BAD_UTF8_STRING:
            printf("Bad UTF8 string\n");
            break;
        case MQTTCLIENT_NULL_PARAMETER:
            printf("Null parameter\n");
            break;
        case MQTTCLIENT_TOPICNAME_TRUNCATED:
            printf("Topic name truncated\n");
            break;
        case MQTTCLIENT_BAD_STRUCTURE:
            printf("Bad structure\n");
            break;
        case MQTTCLIENT_BAD_QOS:
            printf("Bad QoS\n");
            break;
        default:
            printf("Connection failed with code %d\n", rc);
    }
    exit(-1);
}
```

### 2. 资源清理

```c
// 确保正确清理资源
void cleanup(MQTTClient client) {
    MQTTClient_disconnect(client, 10000);
    MQTTClient_destroy(&client);
}

// 在程序退出前调用
atexit(cleanup);
```

### 3. 重连机制

```c
int connect_with_retry(MQTTClient client, MQTTClient_connectOptions *conn_opts) {
    int rc;
    int retry_count = 0;
    int max_retries = 5;

    while (retry_count < max_retries) {
        rc = MQTTClient_connect(client, conn_opts);
        if (rc == MQTTCLIENT_SUCCESS) {
            printf("Connected to RobustMQ successfully\n");
            return rc;
        }

        printf("Connection attempt %d failed, retrying...\n", retry_count + 1);
        retry_count++;
        sleep(2); // 等待2秒后重试
    }

    printf("Failed to connect after %d attempts\n", max_retries);
    return rc;
}
```

## MQTT 5.0 支持

Paho C 客户端完整支持 MQTT 5.0 协议特性：

### MQTT 5.0 连接示例

```c
#include "MQTTClient.h"

int main()
{
    MQTTClient client;
    MQTTClient_connectOptions5 conn_opts = MQTTClient_connectOptions5_initializer;
    MQTTProperties props = MQTTProperties_initializer;
    int rc;

    MQTTClient_create(&client, "tcp://localhost:1883", "robustmq_mqtt5_client",
        MQTTCLIENT_PERSISTENCE_NONE, NULL);

    // 设置 MQTT 5.0 连接参数
    conn_opts.keepAliveInterval = 20;
    conn_opts.cleansession = 1;
    conn_opts.MQTTVersion = MQTTVERSION_5;

    // 设置 MQTT 5.0 属性
    MQTTProperty property;
    property.identifier = MQTTPROPERTY_CODE_SESSION_EXPIRY_INTERVAL;
    property.value.integer4 = 3600; // 会话过期时间：1小时
    MQTTProperties_add(&props, &property);

    conn_opts.connectProperties = &props;

    // 连接到 RobustMQ Broker
    if ((rc = MQTTClient_connect(client, &conn_opts)) != MQTTCLIENT_SUCCESS)
    {
        printf("Failed to connect to RobustMQ with MQTT 5.0, return code %d\n", rc);
        exit(-1);
    }

    printf("Connected to RobustMQ with MQTT 5.0 successfully\n");

    // 清理属性
    MQTTProperties_free(&props);

    // ... 其他操作 ...

    MQTTClient_disconnect(client, 10000);
    MQTTClient_destroy(&client);
    return rc;
}
```

## 常见问题

### Q: 如何处理连接断开？

A: 设置连接丢失回调函数：

```c
void connectionLost(void *context, char *cause)
{
    printf("Connection lost: %s\n", cause);
    printf("Reconnecting...\n");
    // 实现重连逻辑
}

MQTTClient_setCallbacks(client, NULL, connectionLost, messageArrived, NULL);
```

### Q: 如何设置消息质量等级 (QoS)？

A: RobustMQ 支持 MQTT 标准的三种 QoS 等级：

- **QoS 0**: 最多一次传递
- **QoS 1**: 至少一次传递
- **QoS 2**: 恰好一次传递

```c
// 设置不同的 QoS 等级
pubmsg.qos = 0;  // QoS 0
pubmsg.qos = 1;  // QoS 1
pubmsg.qos = 2;  // QoS 2
```

### Q: 如何处理大消息？

A: 对于大消息，建议：

1. 检查 RobustMQ 的最大消息大小限制
2. 考虑消息分片传输
3. 使用适当的 QoS 等级确保可靠传输

## 性能优化建议

### 1. 连接池

对于高频率的消息发送，考虑使用连接池：

```c
// 维护多个连接实例
MQTTClient clients[POOL_SIZE];
int current_client = 0;

MQTTClient get_client() {
    current_client = (current_client + 1) % POOL_SIZE;
    return clients[current_client];
}
```

### 2. 批量操作

```c
// 批量发布消息
void publish_batch(MQTTClient client, char* topics[], char* payloads[], int count) {
    for (int i = 0; i < count; i++) {
        MQTTClient_message pubmsg = MQTTClient_message_initializer;
        pubmsg.payload = payloads[i];
        pubmsg.payloadlen = strlen(payloads[i]);
        pubmsg.qos = 1;
        pubmsg.retained = 0;

        MQTTClient_deliveryToken token;
        MQTTClient_publishMessage(client, topics[i], &pubmsg, &token);
    }
}
```

### 3. 异步处理

对于高性能场景，推荐使用异步 API：

```c
#include "MQTTAsync.h"

void onConnect(void* context, MQTTAsync_successData* response)
{
    printf("Successfully connected to RobustMQ\n");
}

void onConnectFailure(void* context, MQTTAsync_failureData* response)
{
    printf("Connect failed, rc %d\n", response->code);
}

int main()
{
    MQTTAsync client;
    MQTTAsync_connectOptions conn_opts = MQTTAsync_connectOptions_initializer;

    MQTTAsync_create(&client, "tcp://localhost:1883", "robustmq_async_client",
        MQTTCLIENT_PERSISTENCE_NONE, NULL);

    conn_opts.keepAliveInterval = 20;
    conn_opts.cleansession = 1;
    conn_opts.onSuccess = onConnect;
    conn_opts.onFailure = onConnectFailure;
    conn_opts.context = client;

    if ((rc = MQTTAsync_connect(client, &conn_opts)) != MQTTASYNC_SUCCESS)
    {
        printf("Failed to start connect, return code %d\n", rc);
        exit(-1);
    }

    // ... 异步处理逻辑 ...

    return 0;
}
```

## 总结

使用 Eclipse Paho C 客户端连接 RobustMQ MQTT Broker 非常简单直接。该客户端库功能完整，支持 MQTT 3.1.1 和 MQTT 5.0 协议，提供了同步和异步两种 API，能够满足从嵌入式设备到服务器应用的各种使用场景。

通过合理使用连接池、批量操作和异步处理等优化技术，可以在保证可靠性的同时获得优秀的性能表现。
