# 使用 Java SDK 连接 RobustMQ

## 概述

Eclipse Paho Java Client 是用 Java 编写的 MQTT 客户端库，可用于 JVM 或其他 Java 兼容平台（例如 Android）连接 RobustMQ MQTT Broker。

Eclipse Paho Java Client 提供了 `MqttAsyncClient` 和 `MqttClient` 异步和同步两种 API，适用于不同的应用场景。

## 安装依赖

### 通过 Maven 安装

在 `pom.xml` 中添加以下依赖：

```xml
<dependency>
    <groupId>org.eclipse.paho</groupId>
    <artifactId>org.eclipse.paho.client.mqttv3</artifactId>
    <version>1.2.5</version>
</dependency>
```

### 通过 Gradle 安装

在 `build.gradle` 中添加以下依赖：

```groovy
implementation 'org.eclipse.paho:org.eclipse.paho.client.mqttv3:1.2.5'
```

## 基础连接示例

### 发布和订阅示例

**App.java**

```java
package io.robustmq;

import org.eclipse.paho.client.mqttv3.MqttClient;
import org.eclipse.paho.client.mqttv3.MqttConnectOptions;
import org.eclipse.paho.client.mqttv3.MqttException;
import org.eclipse.paho.client.mqttv3.MqttMessage;
import org.eclipse.paho.client.mqttv3.persist.MemoryPersistence;

public class App {
    public static void main(String[] args) {
        String subTopic = "robustmq/test/#";
        String pubTopic = "robustmq/test/hello";
        String content = "Hello RobustMQ!";
        int qos = 2;
        String broker = "tcp://localhost:1883";
        String clientId = "robustmq_java_client";
        MemoryPersistence persistence = new MemoryPersistence();

        try {
            MqttClient client = new MqttClient(broker, clientId, persistence);

            // MQTT 连接选项
            MqttConnectOptions connOpts = new MqttConnectOptions();
            connOpts.setUserName("your_username");
            connOpts.setPassword("your_password".toCharArray());
            // 保留会话
            connOpts.setCleanSession(true);
            // 设置连接超时
            connOpts.setConnectionTimeout(10);
            // 设置心跳间隔
            connOpts.setKeepAliveInterval(20);

            // 设置回调
            client.setCallback(new RobustMQCallback());

            // 建立连接
            System.out.println("Connecting to RobustMQ broker: " + broker);
            client.connect(connOpts);

            System.out.println("Connected to RobustMQ successfully");
            System.out.println("Publishing message: " + content);

            // 订阅主题
            client.subscribe(subTopic, qos);
            System.out.println("Subscribed to topic: " + subTopic);

            // 消息发布所需参数
            MqttMessage message = new MqttMessage(content.getBytes());
            message.setQos(qos);
            message.setRetained(false);

            client.publish(pubTopic, message);
            System.out.println("Message published to topic: " + pubTopic);

            // 等待一段时间以接收消息
            Thread.sleep(2000);

            // 断开连接
            client.disconnect();
            System.out.println("Disconnected from RobustMQ");
            client.close();

        } catch (MqttException me) {
            System.out.println("Reason: " + me.getReasonCode());
            System.out.println("Message: " + me.getMessage());
            System.out.println("Localized message: " + me.getLocalizedMessage());
            System.out.println("Cause: " + me.getCause());
            System.out.println("Exception: " + me);
            me.printStackTrace();
        } catch (InterruptedException e) {
            e.printStackTrace();
        }
    }
}
```

### 回调消息处理类

**RobustMQCallback.java**

```java
package io.robustmq;

import org.eclipse.paho.client.mqttv3.IMqttDeliveryToken;
import org.eclipse.paho.client.mqttv3.MqttCallback;
import org.eclipse.paho.client.mqttv3.MqttMessage;

public class RobustMQCallback implements MqttCallback {

    @Override
    public void connectionLost(Throwable cause) {
        // 连接丢失后，一般在这里面进行重连
        System.out.println("Connection lost to RobustMQ: " + cause.getMessage());
        System.out.println("Attempting to reconnect...");
        // 实现重连逻辑
    }

    @Override
    public void messageArrived(String topic, MqttMessage message) throws Exception {
        // 订阅后得到的消息会执行到这里
        System.out.println("=== Message Received ===");
        System.out.println("Topic: " + topic);
        System.out.println("QoS: " + message.getQos());
        System.out.println("Retained: " + message.isRetained());
        System.out.println("Message: " + new String(message.getPayload()));
        System.out.println("========================");
    }

    @Override
    public void deliveryComplete(IMqttDeliveryToken token) {
        System.out.println("Message delivery complete: " + token.isComplete());
        try {
            System.out.println("Message ID: " + token.getMessageId());
            if (token.getTopics() != null) {
                System.out.println("Published to topics: " + String.join(", ", token.getTopics()));
            }
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
```

## 高级功能

### SSL/TLS 连接

```java
import org.eclipse.paho.client.mqttv3.MqttClient;
import org.eclipse.paho.client.mqttv3.MqttConnectOptions;
import javax.net.ssl.SSLContext;
import javax.net.ssl.TrustManagerFactory;
import java.security.KeyStore;
import java.io.FileInputStream;

public class SSLConnection {
    public static void main(String[] args) {
        String broker = "ssl://localhost:1884";
        String clientId = "robustmq_ssl_client";

        try {
            MqttClient client = new MqttClient(broker, clientId);
            MqttConnectOptions connOpts = new MqttConnectOptions();

            // 配置 SSL
            SSLContext sslContext = createSSLContext();
            connOpts.setSocketFactory(sslContext.getSocketFactory());

            // 其他连接参数
            connOpts.setCleanSession(true);
            connOpts.setConnectionTimeout(10);
            connOpts.setKeepAliveInterval(20);

            System.out.println("Connecting to RobustMQ with SSL...");
            client.connect(connOpts);
            System.out.println("Connected to RobustMQ with SSL successfully");

            // ... 其他操作 ...

            client.disconnect();
            client.close();

        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    private static SSLContext createSSLContext() throws Exception {
        // 加载信任证书
        KeyStore trustStore = KeyStore.getInstance(KeyStore.getDefaultType());
        trustStore.load(new FileInputStream("/path/to/truststore.jks"), "password".toCharArray());

        TrustManagerFactory trustManagerFactory = TrustManagerFactory.getInstance(TrustManagerFactory.getDefaultAlgorithm());
        trustManagerFactory.init(trustStore);

        SSLContext sslContext = SSLContext.getInstance("TLS");
        sslContext.init(null, trustManagerFactory.getTrustManagers(), null);

        return sslContext;
    }
}
```

### 异步客户端示例

```java
import org.eclipse.paho.client.mqttv3.*;

public class AsyncMQTTClient {
    public static void main(String[] args) {
        String broker = "tcp://localhost:1883";
        String clientId = "robustmq_async_client";

        try {
            MqttAsyncClient client = new MqttAsyncClient(broker, clientId);

            MqttConnectOptions connOpts = new MqttConnectOptions();
            connOpts.setCleanSession(true);
            connOpts.setKeepAliveInterval(20);

            // 设置回调
            client.setCallback(new RobustMQCallback());

            // 异步连接
            IMqttToken connectToken = client.connect(connOpts);
            connectToken.waitForCompletion();

            System.out.println("Connected to RobustMQ asynchronously");

            // 异步订阅
            String topic = "robustmq/async/test";
            IMqttToken subToken = client.subscribe(topic, 1);
            subToken.waitForCompletion();

            System.out.println("Subscribed to topic: " + topic);

            // 异步发布
            String message = "Async message from RobustMQ Java client";
            MqttMessage mqttMessage = new MqttMessage(message.getBytes());
            mqttMessage.setQos(1);

            IMqttDeliveryToken pubToken = client.publish(topic, mqttMessage);
            pubToken.waitForCompletion();

            System.out.println("Message published asynchronously");

            // 等待消息
            Thread.sleep(2000);

            // 断开连接
            client.disconnect();
            client.close();

        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
```

### 连接池实现

```java
import org.eclipse.paho.client.mqttv3.MqttClient;
import org.eclipse.paho.client.mqttv3.MqttConnectOptions;
import java.util.concurrent.BlockingQueue;
import java.util.concurrent.LinkedBlockingQueue;

public class MQTTConnectionPool {
    private final BlockingQueue<MqttClient> pool;
    private final String broker;
    private final int poolSize;

    public MQTTConnectionPool(String broker, int poolSize) {
        this.broker = broker;
        this.poolSize = poolSize;
        this.pool = new LinkedBlockingQueue<>();
        initializePool();
    }

    private void initializePool() {
        try {
            for (int i = 0; i < poolSize; i++) {
                String clientId = "robustmq_pool_client_" + i;
                MqttClient client = new MqttClient(broker, clientId);

                MqttConnectOptions connOpts = new MqttConnectOptions();
                connOpts.setCleanSession(true);
                connOpts.setKeepAliveInterval(20);
                connOpts.setAutomaticReconnect(true);

                client.connect(connOpts);
                pool.offer(client);
            }
            System.out.println("Connection pool initialized with " + poolSize + " connections");
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    public MqttClient getConnection() throws InterruptedException {
        return pool.take();
    }

    public void returnConnection(MqttClient client) {
        if (client != null && client.isConnected()) {
            pool.offer(client);
        }
    }

    public void closePool() {
        while (!pool.isEmpty()) {
            try {
                MqttClient client = pool.poll();
                if (client != null && client.isConnected()) {
                    client.disconnect();
                    client.close();
                }
            } catch (Exception e) {
                e.printStackTrace();
            }
        }
    }
}
```

## MQTT 5.0 支持

### MQTT 5.0 连接示例

```java
import org.eclipse.paho.mqttv5.client.MqttClient;
import org.eclipse.paho.mqttv5.client.MqttConnectionOptions;
import org.eclipse.paho.mqttv5.common.MqttException;
import org.eclipse.paho.mqttv5.common.MqttMessage;
import org.eclipse.paho.mqttv5.common.packet.MqttProperties;

public class MQTT5Client {
    public static void main(String[] args) {
        String broker = "tcp://localhost:1883";
        String clientId = "robustmq_mqtt5_client";

        try {
            MqttClient client = new MqttClient(broker, clientId);

            // MQTT 5.0 连接选项
            MqttConnectionOptions connOpts = new MqttConnectionOptions();
            connOpts.setCleanStart(true);
            connOpts.setKeepAliveInterval(20);

            // 设置 MQTT 5.0 属性
            MqttProperties connectProperties = new MqttProperties();
            connectProperties.setSessionExpiryInterval(3600L); // 会话过期时间：1小时
            connectProperties.setReceiveMaximum(100); // 接收最大值
            connOpts.setConnectionProperties(connectProperties);

            // 设置回调
            client.setCallback(new MQTT5Callback());

            // 连接到 RobustMQ
            System.out.println("Connecting to RobustMQ with MQTT 5.0...");
            client.connect(connOpts);
            System.out.println("Connected to RobustMQ with MQTT 5.0 successfully");

            // 订阅主题
            String topic = "robustmq/mqtt5/test";
            client.subscribe(topic, 1);

            // 发布消息
            String content = "MQTT 5.0 message from RobustMQ";
            MqttMessage message = new MqttMessage(content.getBytes());
            message.setQos(1);

            // 设置发布属性
            MqttProperties pubProperties = new MqttProperties();
            pubProperties.setMessageExpiryInterval(300L); // 消息过期时间：5分钟
            pubProperties.setPayloadFormat(true); // 载荷格式指示器

            client.publish(topic, message, null, null, pubProperties);
            System.out.println("MQTT 5.0 message published");

            // 等待消息
            Thread.sleep(2000);

            client.disconnect();
            client.close();

        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
```

## 高级功能

### 自动重连机制

```java
import org.eclipse.paho.client.mqttv3.*;

public class AutoReconnectClient {
    private MqttClient client;
    private MqttConnectOptions connOpts;
    private String broker;
    private String clientId;

    public AutoReconnectClient(String broker, String clientId) {
        this.broker = broker;
        this.clientId = clientId;
        setupConnection();
    }

    private void setupConnection() {
        try {
            client = new MqttClient(broker, clientId);

            connOpts = new MqttConnectOptions();
            connOpts.setCleanSession(true);
            connOpts.setAutomaticReconnect(true); // 启用自动重连
            connOpts.setMaxReconnectDelay(30000); // 最大重连延迟：30秒
            connOpts.setKeepAliveInterval(20);

            client.setCallback(new MqttCallback() {
                @Override
                public void connectionLost(Throwable cause) {
                    System.out.println("Connection lost: " + cause.getMessage());
                    System.out.println("Automatic reconnection will be attempted...");
                }

                @Override
                public void messageArrived(String topic, MqttMessage message) {
                    System.out.println("Message received on topic: " + topic);
                    System.out.println("Content: " + new String(message.getPayload()));
                }

                @Override
                public void deliveryComplete(IMqttDeliveryToken token) {
                    System.out.println("Delivery complete for message ID: " + token.getMessageId());
                }
            });

        } catch (MqttException e) {
            e.printStackTrace();
        }
    }

    public void connect() throws MqttException {
        if (!client.isConnected()) {
            System.out.println("Connecting to RobustMQ...");
            client.connect(connOpts);
            System.out.println("Connected to RobustMQ successfully");
        }
    }

    public void subscribe(String topic, int qos) throws MqttException {
        client.subscribe(topic, qos);
        System.out.println("Subscribed to topic: " + topic);
    }

    public void publish(String topic, String content, int qos) throws MqttException {
        MqttMessage message = new MqttMessage(content.getBytes());
        message.setQos(qos);
        client.publish(topic, message);
        System.out.println("Published message to topic: " + topic);
    }

    public void disconnect() throws MqttException {
        if (client.isConnected()) {
            client.disconnect();
            client.close();
            System.out.println("Disconnected from RobustMQ");
        }
    }
}
```

### 批量消息处理

```java
import org.eclipse.paho.client.mqttv3.*;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

public class BatchMessageProcessor {
    private final MqttClient client;
    private final ExecutorService executor;

    public BatchMessageProcessor(String broker, String clientId) throws MqttException {
        this.client = new MqttClient(broker, clientId);
        this.executor = Executors.newFixedThreadPool(10);

        MqttConnectOptions connOpts = new MqttConnectOptions();
        connOpts.setCleanSession(true);
        connOpts.setKeepAliveInterval(20);

        client.connect(connOpts);
    }

    // 批量发布消息
    public void publishBatch(List<MessageData> messages) {
        messages.forEach(msgData -> {
            CompletableFuture.runAsync(() -> {
                try {
                    MqttMessage message = new MqttMessage(msgData.getContent().getBytes());
                    message.setQos(msgData.getQos());
                    message.setRetained(msgData.isRetained());

                    client.publish(msgData.getTopic(), message);
                    System.out.println("Published: " + msgData.getTopic());
                } catch (MqttException e) {
                    e.printStackTrace();
                }
            }, executor);
        });
    }

    public void close() throws MqttException {
        executor.shutdown();
        client.disconnect();
        client.close();
    }

    // 消息数据类
    public static class MessageData {
        private String topic;
        private String content;
        private int qos;
        private boolean retained;

        public MessageData(String topic, String content, int qos, boolean retained) {
            this.topic = topic;
            this.content = content;
            this.qos = qos;
            this.retained = retained;
        }

        // Getters
        public String getTopic() { return topic; }
        public String getContent() { return content; }
        public int getQos() { return qos; }
        public boolean isRetained() { return retained; }
    }
}
```

## 连接参数配置

### 基础连接参数

| 参数                | 说明                 | 默认值               |
| ------------------- | -------------------- | -------------------- |
| `broker`            | RobustMQ Broker 地址 | tcp://localhost:1883 |
| `clientId`          | 客户端唯一标识       | 自动生成             |
| `cleanSession`      | 是否清除会话         | true                 |
| `keepAliveInterval` | 心跳间隔（秒）       | 60                   |
| `connectionTimeout` | 连接超时（秒）       | 30                   |

### RobustMQ 支持的协议端口

| 协议                | 端口 | 说明                |
| ------------------- | ---- | ------------------- |
| MQTT                | 1883 | 标准 MQTT 端口      |
| MQTT over SSL       | 1884 | 加密 MQTT 连接      |
| MQTT over WebSocket | 8083 | WebSocket 连接      |
| MQTT over WSS       | 8084 | 加密 WebSocket 连接 |

## 最佳实践

### 1. 异常处理

```java
public class RobustMQTTClient {
    private MqttClient client;

    public void connectWithRetry(String broker, String clientId, int maxRetries) {
        int retryCount = 0;

        while (retryCount < maxRetries) {
            try {
                client = new MqttClient(broker, clientId);
                MqttConnectOptions connOpts = new MqttConnectOptions();
                connOpts.setCleanSession(true);
                connOpts.setConnectionTimeout(10);

                client.connect(connOpts);
                System.out.println("Connected to RobustMQ successfully");
                return;

            } catch (MqttException e) {
                retryCount++;
                System.out.println("Connection attempt " + retryCount + " failed: " + e.getMessage());

                if (retryCount < maxRetries) {
                    try {
                        Thread.sleep(2000); // 等待2秒后重试
                    } catch (InterruptedException ie) {
                        Thread.currentThread().interrupt();
                        break;
                    }
                }
            }
        }

        System.out.println("Failed to connect after " + maxRetries + " attempts");
    }
}
```

### 2. 消息持久化

```java
import org.eclipse.paho.client.mqttv3.persist.MqttDefaultFilePersistence;

public class PersistentClient {
    public static void main(String[] args) {
        String broker = "tcp://localhost:1883";
        String clientId = "robustmq_persistent_client";

        try {
            // 使用文件持久化
            String tmpDir = System.getProperty("java.io.tmpdir");
            MqttDefaultFilePersistence persistence = new MqttDefaultFilePersistence(tmpDir);

            MqttClient client = new MqttClient(broker, clientId, persistence);

            MqttConnectOptions connOpts = new MqttConnectOptions();
            connOpts.setCleanSession(false); // 保留会话
            connOpts.setKeepAliveInterval(20);

            client.connect(connOpts);
            System.out.println("Connected with persistent session");

            // ... 其他操作 ...

        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
```

### 3. 性能监控

```java
import java.util.concurrent.atomic.AtomicLong;

public class PerformanceMonitor {
    private final AtomicLong messagesPublished = new AtomicLong(0);
    private final AtomicLong messagesReceived = new AtomicLong(0);
    private final long startTime;

    public PerformanceMonitor() {
        this.startTime = System.currentTimeMillis();
    }

    public void incrementPublished() {
        messagesPublished.incrementAndGet();
    }

    public void incrementReceived() {
        messagesReceived.incrementAndGet();
    }

    public void printStatistics() {
        long elapsed = System.currentTimeMillis() - startTime;
        long published = messagesPublished.get();
        long received = messagesReceived.get();

        System.out.println("=== Performance Statistics ===");
        System.out.println("Elapsed time: " + elapsed + "ms");
        System.out.println("Messages published: " + published);
        System.out.println("Messages received: " + received);
        System.out.println("Publish rate: " + (published * 1000 / elapsed) + " msg/s");
        System.out.println("Receive rate: " + (received * 1000 / elapsed) + " msg/s");
        System.out.println("==============================");
    }
}
```

## Spring Boot 集成

### 配置类

```java
@Configuration
@EnableConfigurationProperties(MQTTProperties.class)
public class MQTTConfig {

    @Autowired
    private MQTTProperties mqttProperties;

    @Bean
    public MqttClient mqttClient() throws MqttException {
        MqttClient client = new MqttClient(
            mqttProperties.getBroker(),
            mqttProperties.getClientId()
        );

        MqttConnectOptions connOpts = new MqttConnectOptions();
        connOpts.setCleanSession(mqttProperties.isCleanSession());
        connOpts.setKeepAliveInterval(mqttProperties.getKeepAliveInterval());
        connOpts.setAutomaticReconnect(true);

        if (mqttProperties.getUsername() != null) {
            connOpts.setUserName(mqttProperties.getUsername());
            connOpts.setPassword(mqttProperties.getPassword().toCharArray());
        }

        client.connect(connOpts);
        return client;
    }
}

@ConfigurationProperties(prefix = "robustmq.mqtt")
@Data
public class MQTTProperties {
    private String broker = "tcp://localhost:1883";
    private String clientId = "robustmq_springboot_client";
    private boolean cleanSession = true;
    private int keepAliveInterval = 20;
    private String username;
    private String password;
}
```

### 服务类

```java
@Service
public class MQTTService {

    @Autowired
    private MqttClient mqttClient;

    public void publish(String topic, String message, int qos) throws MqttException {
        MqttMessage mqttMessage = new MqttMessage(message.getBytes());
        mqttMessage.setQos(qos);
        mqttClient.publish(topic, mqttMessage);
    }

    public void subscribe(String topic, int qos) throws MqttException {
        mqttClient.subscribe(topic, qos);
    }

    @PreDestroy
    public void cleanup() throws MqttException {
        if (mqttClient.isConnected()) {
            mqttClient.disconnect();
            mqttClient.close();
        }
    }
}
```

## 常见问题

### Q: 如何处理连接断开？

A: 使用自动重连和回调函数：

```java
connOpts.setAutomaticReconnect(true);
client.setCallback(new MqttCallback() {
    @Override
    public void connectionLost(Throwable cause) {
        System.out.println("Connection lost, will auto-reconnect: " + cause.getMessage());
    }
    // ... 其他回调方法
});
```

### Q: 如何设置消息质量等级 (QoS)？

A: RobustMQ 支持 MQTT 标准的三种 QoS 等级：

```java
MqttMessage message = new MqttMessage("Hello".getBytes());
message.setQos(0);  // QoS 0: 最多一次传递
message.setQos(1);  // QoS 1: 至少一次传递
message.setQos(2);  // QoS 2: 恰好一次传递
```

### Q: 如何处理大量并发连接？

A: 使用连接池和线程池：

1. 实现连接池管理多个 MQTT 连接
2. 使用线程池处理消息发布和订阅
3. 合理配置连接参数避免资源浪费

## 编译和运行

### Maven 项目结构

```
src/
├── main/
│   ├── java/
│   │   └── io/
│   │       └── robustmq/
│   │           ├── App.java
│   │           └── RobustMQCallback.java
│   └── resources/
│       └── application.properties
└── pom.xml
```

### 编译运行

```bash
# 编译项目
mvn compile

# 运行示例
mvn exec:java -Dexec.mainClass="io.robustmq.App"

# 打包
mvn package
```

## 总结

Eclipse Paho Java Client 是 Java 生态中稳定、广泛应用的 MQTT 客户端库。通过本文档的示例，您可以快速上手使用 Java 连接 RobustMQ MQTT Broker，并实现消息的发布和订阅。

该客户端库支持 MQTT 3.1.1 和 MQTT 5.0 协议，提供了丰富的配置选项和高级功能，能够满足从简单的 IoT 应用到复杂的企业级系统的各种需求。

通过合理使用连接池、异步处理、自动重连等功能，可以构建高性能、高可靠性的 MQTT 应用程序。
