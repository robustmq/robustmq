# Connecting to RobustMQ with C SDK

## Overview

Eclipse Paho C is a full-featured MQTT C client library written in ANSI C, which can be used to connect to RobustMQ MQTT Broker. This client library provides both synchronous and asynchronous APIs for different application scenarios.

## Client Library Introduction

### Eclipse Paho C vs Eclipse Paho Embedded C

- **Eclipse Paho C**: Full-featured MQTT client, suitable for desktop and server environments
- **Eclipse Paho Embedded C**: Lightweight version, mainly targeting embedded environments like mbed, Arduino, and FreeRTOS

### API Types

The client provides two types of APIs:

- **Synchronous API** (MQTTClient):
  - Simpler and easier to use, some calls will block until operation completes
  - Suitable for main thread environments, easier to program
  
- **Asynchronous API** (MQTTAsync):
  - Only one blocking call `waitForCompletion`
  - Result notification through callback functions
  - More suitable for non-main thread environments

## Installing Dependencies

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

### Build from Source

```bash
git clone https://github.com/eclipse/paho.mqtt.c.git
cd paho.mqtt.c
mkdir build
cd build
cmake ..
make
sudo make install
```

## Connection Examples

### Basic Connection and Publishing Example

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

    // Create MQTT client
    MQTTClient_create(&client, ADDRESS, CLIENTID,
        MQTTCLIENT_PERSISTENCE_NONE, NULL);
  
    // Set connection parameters
    conn_opts.keepAliveInterval = 20;
    conn_opts.cleansession = 1;

    // Connect to RobustMQ Broker
    if ((rc = MQTTClient_connect(client, &conn_opts)) != MQTTCLIENT_SUCCESS)
    {
        printf("Failed to connect to RobustMQ, return code %d\n", rc);
        exit(-1);
    }
    
    printf("Connected to RobustMQ successfully\n");
  
    // Publish message
    pubmsg.payload = PAYLOAD;
    pubmsg.payloadlen = strlen(PAYLOAD);
    pubmsg.qos = QOS;
    pubmsg.retained = 0;
    
    MQTTClient_publishMessage(client, TOPIC, &pubmsg, &token);
    printf("Publishing message: %s\n", PAYLOAD);
    printf("On topic: %s\n", TOPIC);
    
    rc = MQTTClient_waitForCompletion(client, token, TIMEOUT);
    printf("Message with delivery token %d delivered\n", token);
  
    // Disconnect
    MQTTClient_disconnect(client, 10000);
    MQTTClient_destroy(&client);
    
    printf("Disconnected from RobustMQ\n");
    return rc;
}
```

### Message Subscription Example

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

// Message arrived callback function
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

// Connection lost callback function
void connectionLost(void *context, char *cause)
{
    printf("Connection lost: %s\n", cause);
}

int main(int argc, char* argv[])
{
    MQTTClient client;
    MQTTClient_connectOptions conn_opts = MQTTClient_connectOptions_initializer;
    int rc;

    // Create MQTT client
    MQTTClient_create(&client, ADDRESS, CLIENTID,
        MQTTCLIENT_PERSISTENCE_NONE, NULL);
    
    // Set callback functions
    MQTTClient_setCallbacks(client, NULL, connectionLost, messageArrived, NULL);

    // Set connection parameters
    conn_opts.keepAliveInterval = 20;
    conn_opts.cleansession = 1;

    // Connect to RobustMQ Broker
    if ((rc = MQTTClient_connect(client, &conn_opts)) != MQTTCLIENT_SUCCESS)
    {
        printf("Failed to connect to RobustMQ, return code %d\n", rc);
        exit(-1);
    }
    
    printf("Connected to RobustMQ successfully\n");

    // Subscribe to topic
    printf("Subscribing to topic: %s\n", TOPIC);
    MQTTClient_subscribe(client, TOPIC, QOS);

    // Wait for messages
    printf("Waiting for messages...\n");
    printf("Press Q<Enter> to quit\n\n");
    
    int ch;
    do {
        ch = getchar();
    } while (ch != 'Q' && ch != 'q');

    // Unsubscribe and disconnect
    MQTTClient_unsubscribe(client, TOPIC);
    MQTTClient_disconnect(client, 10000);
    MQTTClient_destroy(&client);
    
    printf("Disconnected from RobustMQ\n");
    return rc;
}
```

## Advanced Features

### SSL/TLS Connection

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

    // Configure SSL options
    ssl_opts.enableServerCertAuth = 1;
    ssl_opts.trustStore = "/path/to/ca.crt";  // CA certificate path
    ssl_opts.keyStore = "/path/to/client.crt"; // Client certificate path
    ssl_opts.privateKey = "/path/to/client.key"; // Client private key path

    // Set connection parameters
    conn_opts.keepAliveInterval = 20;
    conn_opts.cleansession = 1;
    conn_opts.ssl = &ssl_opts;

    // Connect to RobustMQ Broker (SSL)
    if ((rc = MQTTClient_connect(client, &conn_opts)) != MQTTCLIENT_SUCCESS)
    {
        printf("Failed to connect to RobustMQ with SSL, return code %d\n", rc);
        exit(-1);
    }
    
    printf("Connected to RobustMQ with SSL successfully\n");

    // ... other operations ...

    MQTTClient_disconnect(client, 10000);
    MQTTClient_destroy(&client);
    return rc;
}
```

### Username/Password Authentication

```c
#include "MQTTClient.h"

int main()
{
    MQTTClient client;
    MQTTClient_connectOptions conn_opts = MQTTClient_connectOptions_initializer;
    int rc;

    MQTTClient_create(&client, "tcp://localhost:1883", "robustmq_auth_client",
        MQTTCLIENT_PERSISTENCE_NONE, NULL);

    // Set authentication information
    conn_opts.keepAliveInterval = 20;
    conn_opts.cleansession = 1;
    conn_opts.username = "your_username";
    conn_opts.password = "your_password";

    // Connect to RobustMQ Broker
    if ((rc = MQTTClient_connect(client, &conn_opts)) != MQTTCLIENT_SUCCESS)
    {
        printf("Authentication failed, return code %d\n", rc);
        exit(-1);
    }
    
    printf("Authenticated and connected to RobustMQ successfully\n");

    // ... other operations ...

    MQTTClient_disconnect(client, 10000);
    MQTTClient_destroy(&client);
    return rc;
}
```

## Compilation and Execution

### Compilation Commands

```bash
# Basic compilation
gcc -o mqtt_client mqtt_client.c -lpaho-mqtt3c

# If using SSL
gcc -o mqtt_ssl_client mqtt_ssl_client.c -lpaho-mqtt3cs

# If using asynchronous API
gcc -o mqtt_async_client mqtt_async_client.c -lpaho-mqtt3a
```

### Running Examples

```bash
# Run publisher client
./mqtt_client

# Run subscriber client
./mqtt_subscriber
```

## Connection Parameters

### Basic Connection Parameters

| Parameter | Description | Default Value |
|-----------|-------------|---------------|
| `ADDRESS` | RobustMQ Broker address | tcp://localhost:1883 |
| `CLIENTID` | Unique client identifier | Auto-generated |
| `keepAliveInterval` | Heartbeat interval (seconds) | 60 |
| `cleansession` | Whether to clear session | 1 |

### RobustMQ Supported Protocol Ports

| Protocol | Port | Description |
|----------|------|-------------|
| MQTT | 1883 | Standard MQTT port |
| MQTT over SSL | 1885 | Encrypted MQTT connection |
| MQTT over WebSocket | 8083 | WebSocket connection |
| MQTT over WSS | 8084 | Encrypted WebSocket connection |

## Best Practices

### 1. Error Handling

```c
// Connection error handling
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

### 2. Resource Cleanup

```c
// Ensure proper resource cleanup
void cleanup(MQTTClient client) {
    MQTTClient_disconnect(client, 10000);
    MQTTClient_destroy(&client);
}

// Call before program exit
atexit(cleanup);
```

### 3. Reconnection Mechanism

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
        sleep(2); // Wait 2 seconds before retry
    }
    
    printf("Failed to connect after %d attempts\n", max_retries);
    return rc;
}
```

## MQTT 5.0 Support

The Paho C client fully supports MQTT 5.0 protocol features:

### MQTT 5.0 Connection Example

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

    // Set MQTT 5.0 connection parameters
    conn_opts.keepAliveInterval = 20;
    conn_opts.cleansession = 1;
    conn_opts.MQTTVersion = MQTTVERSION_5;

    // Set MQTT 5.0 properties
    MQTTProperty property;
    property.identifier = MQTTPROPERTY_CODE_SESSION_EXPIRY_INTERVAL;
    property.value.integer4 = 3600; // Session expiry: 1 hour
    MQTTProperties_add(&props, &property);

    conn_opts.connectProperties = &props;

    // Connect to RobustMQ Broker
    if ((rc = MQTTClient_connect(client, &conn_opts)) != MQTTCLIENT_SUCCESS)
    {
        printf("Failed to connect to RobustMQ with MQTT 5.0, return code %d\n", rc);
        exit(-1);
    }
    
    printf("Connected to RobustMQ with MQTT 5.0 successfully\n");

    // Clean up properties
    MQTTProperties_free(&props);

    // ... other operations ...

    MQTTClient_disconnect(client, 10000);
    MQTTClient_destroy(&client);
    return rc;
}
```

## Frequently Asked Questions

### Q: How to handle connection disconnections?

A: Set connection lost callback function:

```c
void connectionLost(void *context, char *cause)
{
    printf("Connection lost: %s\n", cause);
    printf("Reconnecting...\n");
    // Implement reconnection logic
}

MQTTClient_setCallbacks(client, NULL, connectionLost, messageArrived, NULL);
```

### Q: How to set Quality of Service (QoS) levels?

A: RobustMQ supports the three standard MQTT QoS levels:

- **QoS 0**: At most once delivery
- **QoS 1**: At least once delivery  
- **QoS 2**: Exactly once delivery

```c
// Set different QoS levels
pubmsg.qos = 0;  // QoS 0
pubmsg.qos = 1;  // QoS 1
pubmsg.qos = 2;  // QoS 2
```

### Q: How to handle large messages?

A: For large messages, consider:

1. Check RobustMQ's maximum message size limit
2. Consider message fragmentation
3. Use appropriate QoS levels to ensure reliable transmission

## Performance Optimization Tips

### 1. Connection Pooling

For high-frequency message sending, consider using connection pools:

```c
// Maintain multiple connection instances
MQTTClient clients[POOL_SIZE];
int current_client = 0;

MQTTClient get_client() {
    current_client = (current_client + 1) % POOL_SIZE;
    return clients[current_client];
}
```

### 2. Batch Operations

```c
// Batch publish messages
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

### 3. Asynchronous Processing

For high-performance scenarios, asynchronous API is recommended:

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

    // ... asynchronous processing logic ...
    
    return 0;
}
```

## Summary

Using the Eclipse Paho C client to connect to RobustMQ MQTT Broker is simple and straightforward. This client library is feature-complete, supports both MQTT 3.1.1 and MQTT 5.0 protocols, provides synchronous and asynchronous APIs, and can meet various use cases from embedded devices to server applications.

By properly using optimization techniques such as connection pooling, batch operations, and asynchronous processing, you can achieve excellent performance while ensuring reliability.

