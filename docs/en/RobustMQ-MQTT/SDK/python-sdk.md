# Connecting to RobustMQ with Python SDK

## Overview

Eclipse Paho Python is the Python language client library under the Eclipse Paho project. This library can connect to RobustMQ MQTT Broker to publish messages, subscribe to topics, and receive published messages.

This client library is simple to use, feature-complete, and is one of the most popular MQTT client libraries in the Python ecosystem.

## Installing Dependencies

### Install using pip

```bash
pip install paho-mqtt
```

### Install using conda

```bash
conda install paho-mqtt
```

### Add to requirements.txt

```txt
paho-mqtt==1.6.1
```

## Basic Connection Example

### Publishing and Subscribing Example

```python
import paho.mqtt.client as mqtt
import time
import json

# Connection success callback
def on_connect(client, userdata, flags, rc):
    print(f'Connection result code: {rc}')
    if rc == 0:
        print('Successfully connected to RobustMQ')
        # Subscribe to topic after successful connection
        client.subscribe('robustmq/python/test/#')
        print('Subscribed to topic: robustmq/python/test/#')
    else:
        print(f'Connection failed, return code: {rc}')

# Message received callback
def on_message(client, userdata, msg):
    print('=== Message Received ===')
    print(f'Topic: {msg.topic}')
    print(f'Message: {msg.payload.decode()}')
    print(f'QoS: {msg.qos}')
    print(f'Retained: {msg.retain}')
    print('========================')

# Message published callback
def on_publish(client, userdata, mid):
    print(f'Message published successfully, Message ID: {mid}')

# Subscription success callback
def on_subscribe(client, userdata, mid, granted_qos):
    print(f'Subscription successful, Message ID: {mid}, Granted QoS: {granted_qos}')

# Disconnect callback
def on_disconnect(client, userdata, rc):
    if rc != 0:
        print(f'Unexpected disconnection, return code: {rc}')
    else:
        print('Normal disconnection')

def main():
    # Create client instance
    client = mqtt.Client(client_id='robustmq_python_client')

    # Set username and password
    client.username_pw_set('your_username', 'your_password')

    # Set callback functions
    client.on_connect = on_connect
    client.on_message = on_message
    client.on_publish = on_publish
    client.on_subscribe = on_subscribe
    client.on_disconnect = on_disconnect

    # Connect to RobustMQ Broker
    try:
        print('Connecting to RobustMQ...')
        client.connect('localhost', 1883, 60)

        # Start network loop
        client.loop_start()

        # Wait for connection establishment
        time.sleep(2)

        # Publish test messages
        test_messages = [
            {'topic': 'robustmq/python/test/hello', 'payload': 'Hello RobustMQ!'},
            {'topic': 'robustmq/python/test/data', 'payload': json.dumps({'sensor': 'temp', 'value': 25.5})},
            {'topic': 'robustmq/python/test/status', 'payload': 'online'}
        ]

        for msg in test_messages:
            result = client.publish(msg['topic'], msg['payload'], qos=1)
            if result.rc == mqtt.MQTT_ERR_SUCCESS:
                print(f"Message published to: {msg['topic']}")
            else:
                print(f"Message publish failed: {result.rc}")
            time.sleep(1)

        # Keep connection for a while to receive messages
        print('Waiting to receive messages...')
        time.sleep(5)

    except Exception as e:
        print(f'Connection error: {e}')
    finally:
        # Stop network loop and disconnect
        client.loop_stop()
        client.disconnect()
        print('Disconnected from RobustMQ')

if __name__ == '__main__':
    main()
```

## Advanced Features

### SSL/TLS Connection

```python
import paho.mqtt.client as mqtt
import ssl

def on_connect(client, userdata, flags, rc):
    print(f'SSL connection result: {rc}')
    if rc == 0:
        print('Successfully established SSL connection to RobustMQ')

def main():
    client = mqtt.Client(client_id='robustmq_python_ssl_client')

    # Configure SSL/TLS
    context = ssl.create_default_context(ssl.Purpose.SERVER_AUTH)
    context.check_hostname = False
    context.verify_mode = ssl.CERT_NONE  # For testing; use CERT_REQUIRED in production

    client.tls_set_context(context)

    # Set callback
    client.on_connect = on_connect

    # Connect to RobustMQ SSL port
    try:
        client.connect('localhost', 1885, 60)  # SSL port
        client.loop_start()

        # Publish SSL test message
        client.publish('robustmq/ssl/test', 'SSL connection test', qos=1)

        time.sleep(3)

    except Exception as e:
        print(f'SSL connection error: {e}')
    finally:
        client.loop_stop()
        client.disconnect()

if __name__ == '__main__':
    main()
```

## Connection Parameters

### Basic Connection Parameters

| Parameter | Description | Default Value |
|-----------|-------------|---------------|
| `host` | RobustMQ Broker address | localhost |
| `port` | Connection port | 1883 |
| `client_id` | Unique client identifier | Auto-generated |
| `keepalive` | Heartbeat interval (seconds) | 60 |
| `clean_session` | Whether to clear session | True |

### RobustMQ Supported Protocol Ports

| Protocol | Port | Connection Method |
|----------|------|-------------------|
| MQTT | 1883 | `client.connect('localhost', 1883)` |
| MQTT over SSL | 1885 | `client.connect('localhost', 1885)` + SSL config |
| MQTT over WebSocket | 8083 | `transport="websockets"` |
| MQTT over WSS | 8085 | WebSocket + SSL config |

## Best Practices

### 1. Exception Handling and Reconnection

```python
import paho.mqtt.client as mqtt
import time

class RobustMQTTClient:
    def __init__(self, broker, port, client_id):
        self.broker = broker
        self.port = port
        self.client_id = client_id
        self.client = None
        self.max_retries = 5

    def connect_with_retry(self):
        """Connect with retry mechanism"""
        retry_count = 0

        while retry_count < self.max_retries:
            try:
                self.client = mqtt.Client(client_id=self.client_id)
                self.client.on_connect = self.on_connect
                self.client.on_disconnect = self.on_disconnect

                self.client.connect(self.broker, self.port, 60)
                self.client.loop_start()
                return True

            except Exception as e:
                retry_count += 1
                print(f'Connection attempt {retry_count} failed: {e}')

                if retry_count < self.max_retries:
                    time.sleep(2 ** retry_count)  # Exponential backoff

        print(f'Connection failed after {self.max_retries} attempts')
        return False

    def on_connect(self, client, userdata, flags, rc):
        if rc == 0:
            print('Connection successful')
        else:
            print(f'Connection failed: {rc}')

    def on_disconnect(self, client, userdata, rc):
        if rc != 0:
            print(f'Unexpected disconnect, attempting reconnect: {rc}')
            self.connect_with_retry()
```

## Frequently Asked Questions

### Q: How to handle connection disconnections?

A: Set disconnect callback and auto-reconnection:

```python
def on_disconnect(client, userdata, rc):
    if rc != 0:
        print(f'Unexpected disconnect, will auto-reconnect: {rc}')
        # Implement reconnection logic

client.on_disconnect = on_disconnect
```

### Q: How to set Quality of Service (QoS) levels?

A: RobustMQ supports the three standard MQTT QoS levels:

```python
# QoS 0: At most once delivery
client.publish(topic, payload, qos=0)

# QoS 1: At least once delivery
client.publish(topic, payload, qos=1)

# QoS 2: Exactly once delivery
client.publish(topic, payload, qos=2)
```

### Q: How to handle large volumes of messages?

A: Use multithreading and message queues:

1. Use `client.loop_start()` to enable background threads
2. Implement message queues to buffer peak traffic
3. Use worker thread pools to process messages

## Execution and Deployment

### Project Structure

```
robustmq-python-client/
├── main.py
├── config.py
├── mqtt_client.py
├── requirements.txt
├── mqtt_config.ini
└── README.md
```

### Running Examples

```bash
# Install dependencies
pip install -r requirements.txt

# Run basic example
python main.py

# Run async example
python async_client.py

# Run data pipeline
python data_pipeline.py
```

## Summary

Eclipse Paho Python is the most mature and stable MQTT client library in the Python ecosystem. This library has a simple and easy-to-use API with complete functionality, making it ideal for rapid development of MQTT applications.

Through the examples in this document, you can quickly get started with using Python to connect to RobustMQ MQTT Broker and implement various application scenarios from simple message sending and receiving to complex data processing pipelines. Combined with Python's rich ecosystem, you can easily build powerful IoT and message processing systems.
