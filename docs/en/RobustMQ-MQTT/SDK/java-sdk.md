# Connecting to RobustMQ with Java SDK

## Overview

Eclipse Paho Java Client is an MQTT client library written in Java, which can be used for JVM or other Java-compatible platforms (such as Android) to connect to RobustMQ MQTT Broker.

Eclipse Paho Java Client provides both `MqttAsyncClient` and `MqttClient` asynchronous and synchronous APIs for different application scenarios.

## Installing Dependencies

### Install via Maven

Add the following dependency to your `pom.xml`:

```xml
<dependency>
    <groupId>org.eclipse.paho</groupId>
    <artifactId>org.eclipse.paho.client.mqttv3</artifactId>
    <version>1.2.5</version>
</dependency>
```

### Install via Gradle

Add the following dependency to your `build.gradle`:

```groovy
implementation 'org.eclipse.paho:org.eclipse.paho.client.mqttv3:1.2.5'
```

## Basic Connection Example

### Publishing and Subscribing Example

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

            // MQTT connection options
            MqttConnectOptions connOpts = new MqttConnectOptions();
            connOpts.setUserName("your_username");
            connOpts.setPassword("your_password".toCharArray());
            // Clean session
            connOpts.setCleanSession(true);
            // Set connection timeout
            connOpts.setConnectionTimeout(10);
            // Set keep alive interval
            connOpts.setKeepAliveInterval(20);

            // Set callback
            client.setCallback(new RobustMQCallback());

            // Establish connection
            System.out.println("Connecting to RobustMQ broker: " + broker);
            client.connect(connOpts);

            System.out.println("Connected to RobustMQ successfully");
            System.out.println("Publishing message: " + content);

            // Subscribe to topic
            client.subscribe(subTopic, qos);
            System.out.println("Subscribed to topic: " + subTopic);

            // Message publishing parameters
            MqttMessage message = new MqttMessage(content.getBytes());
            message.setQos(qos);
            message.setRetained(false);

            client.publish(pubTopic, message);
            System.out.println("Message published to topic: " + pubTopic);

            // Wait for some time to receive messages
            Thread.sleep(2000);

            // Disconnect
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

### Callback Message Handler Class

**RobustMQCallback.java**

```java
package io.robustmq;

import org.eclipse.paho.client.mqttv3.IMqttDeliveryToken;
import org.eclipse.paho.client.mqttv3.MqttCallback;
import org.eclipse.paho.client.mqttv3.MqttMessage;

public class RobustMQCallback implements MqttCallback {

    @Override
    public void connectionLost(Throwable cause) {
        // After connection is lost, usually implement reconnection here
        System.out.println("Connection lost to RobustMQ: " + cause.getMessage());
        System.out.println("Attempting to reconnect...");
        // Implement reconnection logic
    }

    @Override
    public void messageArrived(String topic, MqttMessage message) throws Exception {
        // Messages received after subscription will execute here
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

## Advanced Features

### SSL/TLS Connection

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

            // Configure SSL
            SSLContext sslContext = createSSLContext();
            connOpts.setSocketFactory(sslContext.getSocketFactory());

            // Other connection parameters
            connOpts.setCleanSession(true);
            connOpts.setConnectionTimeout(10);
            connOpts.setKeepAliveInterval(20);

            System.out.println("Connecting to RobustMQ with SSL...");
            client.connect(connOpts);
            System.out.println("Connected to RobustMQ with SSL successfully");

            // ... other operations ...

            client.disconnect();
            client.close();

        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    private static SSLContext createSSLContext() throws Exception {
        // Load trust certificate
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

### Asynchronous Client Example

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

            // Set callback
            client.setCallback(new RobustMQCallback());

            // Asynchronous connection
            IMqttToken connectToken = client.connect(connOpts);
            connectToken.waitForCompletion();

            System.out.println("Connected to RobustMQ asynchronously");

            // Asynchronous subscription
            String topic = "robustmq/async/test";
            IMqttToken subToken = client.subscribe(topic, 1);
            subToken.waitForCompletion();

            System.out.println("Subscribed to topic: " + topic);

            // Asynchronous publishing
            String message = "Async message from RobustMQ Java client";
            MqttMessage mqttMessage = new MqttMessage(message.getBytes());
            mqttMessage.setQos(1);

            IMqttDeliveryToken pubToken = client.publish(topic, mqttMessage);
            pubToken.waitForCompletion();

            System.out.println("Message published asynchronously");

            // Wait for messages
            Thread.sleep(2000);

            // Disconnect
            client.disconnect();
            client.close();

        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
```

## MQTT 5.0 Support

### MQTT 5.0 Connection Example

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

            // MQTT 5.0 connection options
            MqttConnectionOptions connOpts = new MqttConnectionOptions();
            connOpts.setCleanStart(true);
            connOpts.setKeepAliveInterval(20);

            // Set MQTT 5.0 properties
            MqttProperties connectProperties = new MqttProperties();
            connectProperties.setSessionExpiryInterval(3600L); // Session expiry: 1 hour
            connectProperties.setReceiveMaximum(100); // Receive maximum
            connOpts.setConnectionProperties(connectProperties);

            // Set callback
            client.setCallback(new MQTT5Callback());

            // Connect to RobustMQ
            System.out.println("Connecting to RobustMQ with MQTT 5.0...");
            client.connect(connOpts);
            System.out.println("Connected to RobustMQ with MQTT 5.0 successfully");

            // Subscribe to topic
            String topic = "robustmq/mqtt5/test";
            client.subscribe(topic, 1);

            // Publish message
            String content = "MQTT 5.0 message from RobustMQ";
            MqttMessage message = new MqttMessage(content.getBytes());
            message.setQos(1);

            // Set publish properties
            MqttProperties pubProperties = new MqttProperties();
            pubProperties.setMessageExpiryInterval(300L); // Message expiry: 5 minutes
            pubProperties.setPayloadFormat(true); // Payload format indicator

            client.publish(topic, message, null, null, pubProperties);
            System.out.println("MQTT 5.0 message published");

            // Wait for messages
            Thread.sleep(2000);

            client.disconnect();
            client.close();

        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
```

## Performance Optimization

### Connection Pooling

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

## Spring Boot Integration

### Configuration Class

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

### Service Class

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

## Connection Parameters

### Basic Connection Parameters

| Parameter           | Description                  | Default Value        |
| ------------------- | ---------------------------- | -------------------- |
| `broker`            | RobustMQ Broker address      | tcp://localhost:1883 |
| `clientId`          | Unique client identifier     | Auto-generated       |
| `cleanSession`      | Whether to clear session     | true                 |
| `keepAliveInterval` | Heartbeat interval (seconds) | 60                   |
| `connectionTimeout` | Connection timeout (seconds) | 30                   |

### RobustMQ Supported Protocol Ports

| Protocol            | Port | Description                    |
| ------------------- | ---- | ------------------------------ |
| MQTT                | 1883 | Standard MQTT port             |
| MQTT over SSL       | 1884 | Encrypted MQTT connection      |
| MQTT over WebSocket | 8083 | WebSocket connection           |
| MQTT over WSS       | 8084 | Encrypted WebSocket connection |

## Best Practices

### 1. Exception Handling

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
                        Thread.sleep(2000); // Wait 2 seconds before retry
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

### 2. Message Persistence

```java
import org.eclipse.paho.client.mqttv3.persist.MqttDefaultFilePersistence;

public class PersistentClient {
    public static void main(String[] args) {
        String broker = "tcp://localhost:1883";
        String clientId = "robustmq_persistent_client";

        try {
            // Use file persistence
            String tmpDir = System.getProperty("java.io.tmpdir");
            MqttDefaultFilePersistence persistence = new MqttDefaultFilePersistence(tmpDir);

            MqttClient client = new MqttClient(broker, clientId, persistence);

            MqttConnectOptions connOpts = new MqttConnectOptions();
            connOpts.setCleanSession(false); // Retain session
            connOpts.setKeepAliveInterval(20);

            client.connect(connOpts);
            System.out.println("Connected with persistent session");

            // ... other operations ...

        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
```

### 3. Performance Monitoring

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

## Frequently Asked Questions

### Q: How to handle connection disconnections?

A: Use automatic reconnection and callback functions:

```java
connOpts.setAutomaticReconnect(true);
client.setCallback(new MqttCallback() {
    @Override
    public void connectionLost(Throwable cause) {
        System.out.println("Connection lost, will auto-reconnect: " + cause.getMessage());
    }
    // ... other callback methods
});
```

### Q: How to set Quality of Service (QoS) levels?

A: RobustMQ supports the three standard MQTT QoS levels:

```java
MqttMessage message = new MqttMessage("Hello".getBytes());
message.setQos(0);  // QoS 0: At most once delivery
message.setQos(1);  // QoS 1: At least once delivery
message.setQos(2);  // QoS 2: Exactly once delivery
```

### Q: How to handle high concurrent connections?

A: Use connection pools and thread pools:

1. Implement connection pool to manage multiple MQTT connections
2. Use thread pools to handle message publishing and subscription
3. Configure connection parameters properly to avoid resource waste

## Compilation and Execution

### Maven Project Structure

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

### Compilation and Execution

```bash
# Compile project
mvn compile

# Run example
mvn exec:java -Dexec.mainClass="io.robustmq.App"

# Package
mvn package
```

## Summary

Eclipse Paho Java Client is a stable and widely-used MQTT client library in the Java ecosystem. Through the examples in this document, you can quickly get started with using Java to connect to RobustMQ MQTT Broker and implement message publishing and subscription.

The client library supports both MQTT 3.1.1 and MQTT 5.0 protocols, provides rich configuration options and advanced features, and can meet various requirements from simple IoT applications to complex enterprise systems.

By properly using features like connection pooling, asynchronous processing, and automatic reconnection, you can build high-performance and highly reliable MQTT applications.
