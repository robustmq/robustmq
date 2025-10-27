# Connecting to RobustMQ with JavaScript SDK

## Overview

MQTT.js is the MQTT client library for JavaScript, which can be used in both Node.js and browser environments to connect to RobustMQ MQTT Broker. This library is currently the most popular MQTT client library in the JavaScript ecosystem, supporting multiple connection methods and complete MQTT 5.0 protocol.

## Installing Dependencies

### Install using npm

```bash
npm install mqtt
```

### Install using yarn

```bash
yarn add mqtt
```

### Use in Browser

```html
<script src="https://unpkg.com/mqtt/dist/mqtt.min.js"></script>
```

## Node.js Environment Example

### Basic Connection and Publish/Subscribe

```javascript
// Using ES6 modules
import mqtt from 'mqtt'
// Or using CommonJS
// const mqtt = require('mqtt')

// Connection options
const options = {
    clean: true, // true: clear session, false: retain session
    connectTimeout: 4000, // Timeout
    // Authentication info
    clientId: 'robustmq_js_client',
    username: 'your_username',
    password: 'your_password',
    keepalive: 60,
    reconnectPeriod: 1000, // Reconnection interval
}

// Connection string, specify connection method through protocol
// ws: Unencrypted WebSocket connection
// wss: Encrypted WebSocket connection  
// mqtt: Unencrypted TCP connection
// mqtts: Encrypted TCP connection
const connectUrl = 'ws://localhost:8083/mqtt'
const client = mqtt.connect(connectUrl, options)

// Connection success event
client.on('connect', () => {
    console.log('Successfully connected to RobustMQ')
    
    // Subscribe to topic
    const topic = 'robustmq/js/test/#'
    client.subscribe(topic, { qos: 1 }, (err) => {
        if (!err) {
            console.log(`Successfully subscribed to topic: ${topic}`)
            
            // Publish test message
            const pubTopic = 'robustmq/js/test/hello'
            const message = 'Hello RobustMQ from JavaScript!'
            
            client.publish(pubTopic, message, { qos: 1, retain: false }, (err) => {
                if (!err) {
                    console.log(`Message published to: ${pubTopic}`)
                } else {
                    console.log('Message publish failed:', err)
                }
            })
        } else {
            console.log('Subscription failed:', err)
        }
    })
})

// Reconnection event
client.on('reconnect', () => {
    console.log('Reconnecting to RobustMQ...')
})

// Error event
client.on('error', (error) => {
    console.log('Connection error:', error)
})

// Message received event
client.on('message', (topic, message, packet) => {
    console.log('=== Message Received ===')
    console.log('Topic:', topic)
    console.log('Message:', message.toString())
    console.log('QoS:', packet.qos)
    console.log('Retained:', packet.retain)
    console.log('========================')
})

// Disconnect event
client.on('disconnect', () => {
    console.log('Disconnected from RobustMQ')
})

// Graceful shutdown
process.on('SIGINT', () => {
    console.log('Received interrupt signal, shutting down...')
    client.end()
    process.exit(0)
})
```

## Browser Environment Example

### HTML Page Example

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RobustMQ JavaScript Client</title>
    <script src="https://unpkg.com/mqtt/dist/mqtt.min.js"></script>
</head>
<body>
    <h1>RobustMQ JavaScript Client Example</h1>
    
    <div>
        <h3>Connection Status</h3>
        <p id="status">Not Connected</p>
        <button id="connectBtn">Connect</button>
        <button id="disconnectBtn">Disconnect</button>
    </div>
    
    <div>
        <h3>Publish Message</h3>
        <input type="text" id="pubTopic" placeholder="Topic" value="robustmq/browser/test">
        <input type="text" id="pubMessage" placeholder="Message content" value="Hello from browser!">
        <button id="publishBtn">Publish</button>
    </div>
    
    <div>
        <h3>Subscribe Topic</h3>
        <input type="text" id="subTopic" placeholder="Subscribe topic" value="robustmq/browser/test">
        <button id="subscribeBtn">Subscribe</button>
        <button id="unsubscribeBtn">Unsubscribe</button>
    </div>
    
    <div>
        <h3>Received Messages</h3>
        <div id="messages" style="border: 1px solid #ccc; height: 200px; overflow-y: auto; padding: 10px;"></div>
    </div>

    <script>
        let client = null
        const statusEl = document.getElementById('status')
        const messagesEl = document.getElementById('messages')
        
        // Connection configuration
        const options = {
            clean: true,
            connectTimeout: 4000,
            clientId: 'robustmq_browser_client_' + Math.random().toString(16).substr(2, 8),
            username: 'test_user',
            password: 'test_pass',
        }
        
        // Connect function
        function connect() {
            const connectUrl = 'ws://localhost:8083/mqtt'
            client = mqtt.connect(connectUrl, options)
            
            client.on('connect', () => {
                statusEl.textContent = 'Connected to RobustMQ'
                statusEl.style.color = 'green'
                console.log('Browser client connected successfully')
            })
            
            client.on('error', (err) => {
                statusEl.textContent = 'Connection error: ' + err.message
                statusEl.style.color = 'red'
                console.error('Connection error:', err)
            })
            
            client.on('reconnect', () => {
                statusEl.textContent = 'Reconnecting...'
                statusEl.style.color = 'orange'
            })
            
            client.on('message', (topic, message, packet) => {
                const msgDiv = document.createElement('div')
                msgDiv.innerHTML = `
                    <strong>Topic:</strong> ${topic}<br>
                    <strong>Message:</strong> ${message.toString()}<br>
                    <strong>Time:</strong> ${new Date().toLocaleTimeString()}<br>
                    <hr>
                `
                messagesEl.insertBefore(msgDiv, messagesEl.firstChild)
                
                // Limit displayed message count
                if (messagesEl.children.length > 20) {
                    messagesEl.removeChild(messagesEl.lastChild)
                }
            })
            
            client.on('disconnect', () => {
                statusEl.textContent = 'Disconnected'
                statusEl.style.color = 'gray'
            })
        }
        
        // Disconnect function
        function disconnect() {
            if (client) {
                client.end()
                client = null
            }
        }
        
        // Publish message function
        function publishMessage() {
            if (!client || !client.connected) {
                alert('Please connect to RobustMQ first')
                return
            }
            
            const topic = document.getElementById('pubTopic').value
            const message = document.getElementById('pubMessage').value
            
            client.publish(topic, message, { qos: 1 }, (err) => {
                if (!err) {
                    console.log('Message published successfully:', topic)
                } else {
                    console.error('Message publish failed:', err)
                }
            })
        }
        
        // Subscribe topic function
        function subscribeTopic() {
            if (!client || !client.connected) {
                alert('Please connect to RobustMQ first')
                return
            }
            
            const topic = document.getElementById('subTopic').value
            
            client.subscribe(topic, { qos: 1 }, (err) => {
                if (!err) {
                    console.log('Subscription successful:', topic)
                } else {
                    console.error('Subscription failed:', err)
                }
            })
        }
        
        // Unsubscribe function
        function unsubscribeTopic() {
            if (!client || !client.connected) {
                alert('Please connect to RobustMQ first')
                return
            }
            
            const topic = document.getElementById('subTopic').value
            
            client.unsubscribe(topic, (err) => {
                if (!err) {
                    console.log('Unsubscribe successful:', topic)
                } else {
                    console.error('Unsubscribe failed:', err)
                }
            })
        }
        
        // Bind events
        document.getElementById('connectBtn').addEventListener('click', connect)
        document.getElementById('disconnectBtn').addEventListener('click', disconnect)
        document.getElementById('publishBtn').addEventListener('click', publishMessage)
        document.getElementById('subscribeBtn').addEventListener('click', subscribeTopic)
        document.getElementById('unsubscribeBtn').addEventListener('click', unsubscribeTopic)
    </script>
</body>
</html>
```

## MQTT 5.0 Support

MQTT.js fully supports MQTT 5.0 protocol:

### MQTT 5.0 Connection Example

```javascript
import mqtt from 'mqtt'

// MQTT 5.0 connection options
const options = {
    protocolVersion: 5, // Specify MQTT 5.0
    clean: true,
    connectTimeout: 4000,
    clientId: 'robustmq_mqtt5_js_client',
    username: 'mqtt5_user',
    password: 'mqtt5_pass',
    
    // MQTT 5.0 specific properties
    properties: {
        sessionExpiryInterval: 3600, // Session expiry: 1 hour
        receiveMaximum: 100, // Receive maximum
        maximumPacketSize: 1048576, // Max packet size: 1MB
        requestResponseInformation: true,
        requestProblemInformation: true
    }
}

const client = mqtt.connect('ws://localhost:8083/mqtt', options)

client.on('connect', (connack) => {
    console.log('MQTT 5.0 connection successful')
    console.log('Server response:', connack)
    
    // MQTT 5.0 subscription with properties
    const subscribeOptions = {
        qos: 1,
        properties: {
            subscriptionIdentifier: 1
        }
    }
    
    client.subscribe('robustmq/mqtt5/test', subscribeOptions, (err, granted) => {
        if (!err) {
            console.log('MQTT 5.0 subscription successful:', granted)
        }
    })
})

client.on('message', (topic, message, packet) => {
    console.log('MQTT 5.0 message received:')
    console.log('Topic:', topic)
    console.log('Message:', message.toString())
    console.log('Packet properties:', packet.properties)
})

// Publish MQTT 5.0 message
function publishMQTT5Message() {
    const topic = 'robustmq/mqtt5/test'
    const message = 'MQTT 5.0 message from JavaScript'
    
    const publishOptions = {
        qos: 1,
        retain: false,
        properties: {
            messageExpiryInterval: 300, // Message expiry: 5 minutes
            payloadFormatIndicator: true,
            contentType: 'application/json',
            responseTopic: 'robustmq/mqtt5/response',
            userProperties: {
                'source': 'javascript_client',
                'version': '1.0'
            }
        }
    }
    
    client.publish(topic, message, publishOptions, (err) => {
        if (!err) {
            console.log('MQTT 5.0 message published successfully')
        } else {
            console.error('MQTT 5.0 message publish failed:', err)
        }
    })
}

// Publish test message after 5 seconds
setTimeout(publishMQTT5Message, 5000)
```

## Connection Parameters

### Basic Connection Parameters

| Parameter | Description | Default Value |
|-----------|-------------|---------------|
| `host` | RobustMQ Broker address | localhost |
| `port` | Connection port | 1883 |
| `clientId` | Unique client identifier | Auto-generated |
| `clean` | Whether to clear session | true |
| `keepalive` | Heartbeat interval (seconds) | 60 |
| `connectTimeout` | Connection timeout (milliseconds) | 30000 |

### RobustMQ Supported Connection Methods

| Protocol | Port | Connection String Example |
|----------|------|---------------------------|
| MQTT (TCP) | 1883 | `mqtt://localhost:1883` |
| MQTTS (SSL) | 1885 | `mqtts://localhost:1885` |
| WebSocket | 8083 | `ws://localhost:8083/mqtt` |
| WebSocket SSL | 8085 | `wss://localhost:8085/mqtt` |

## Best Practices

### 1. Error Handling and Reconnection

```javascript
const client = mqtt.connect('ws://localhost:8083/mqtt', {
    reconnectPeriod: 1000, // Reconnection interval
    connectTimeout: 4000,
    clean: true
})

client.on('error', (err) => {
    console.error('MQTT error:', err)
    
    // Handle different error types
    if (err.code === 'ECONNREFUSED') {
        console.log('Server refused connection, check server status')
    } else if (err.code === 'ENOTFOUND') {
        console.log('Cannot resolve server address')
    }
})

client.on('offline', () => {
    console.log('Client offline')
})

client.on('reconnect', () => {
    console.log('Reconnecting...')
})
```

### 2. Message Caching and Offline Processing

```javascript
class OfflineMessageCache {
    constructor() {
        this.cache = []
        this.maxCacheSize = 1000
    }
    
    addMessage(topic, message, options) {
        if (this.cache.length >= this.maxCacheSize) {
            this.cache.shift() // Remove oldest message
        }
        
        this.cache.push({
            topic,
            message,
            options,
            timestamp: Date.now()
        })
    }
    
    publishCachedMessages(client) {
        console.log(`Publishing ${this.cache.length} cached messages`)
        
        this.cache.forEach(msg => {
            client.publish(msg.topic, msg.message, msg.options)
        })
        
        this.cache = []
    }
}

const messageCache = new OfflineMessageCache()

const client = mqtt.connect('ws://localhost:8083/mqtt')

client.on('connect', () => {
    console.log('Reconnected, publishing cached messages')
    messageCache.publishCachedMessages(client)
})

client.on('offline', () => {
    console.log('Client offline, messages will be cached')
})

// Smart publish function
function smartPublish(topic, message, options = {}) {
    if (client.connected) {
        client.publish(topic, message, options)
    } else {
        console.log('Offline, message cached')
        messageCache.addMessage(topic, message, options)
    }
}
```

## Frequently Asked Questions

### Q: How to connect to RobustMQ in browser?

A: Browsers can only use WebSocket connections:

```javascript
// Browser can only use WebSocket
const client = mqtt.connect('ws://localhost:8083/mqtt', options)
// Or encrypted connection
const client = mqtt.connect('wss://localhost:8085/mqtt', options)
```

### Q: How to handle large volumes of messages?

A: Use message queues and batch processing:

```javascript
const messageBuffer = []
const BATCH_SIZE = 100

client.on('message', (topic, message) => {
    messageBuffer.push({ topic, message: message.toString() })
    
    if (messageBuffer.length >= BATCH_SIZE) {
        processBatch(messageBuffer.splice(0, BATCH_SIZE))
    }
})

function processBatch(messages) {
    console.log(`Batch processing ${messages.length} messages`)
    // Batch processing logic
}
```

### Q: How to set Quality of Service (QoS) levels?

A: Specify QoS when publishing and subscribing:

```javascript
// Set QoS when publishing
client.publish(topic, message, { qos: 1 })

// Set QoS when subscribing
client.subscribe(topic, { qos: 2 })
```

## Project Structure and Deployment

### Node.js Project Structure

```
robustmq-js-client/
├── package.json
├── src/
│   ├── mqtt-client.js
│   ├── config.js
│   └── utils.js
├── examples/
│   ├── basic-client.js
│   ├── react-example/
│   └── vue-example/
└── README.md
```

### package.json Example

```json
{
  "name": "robustmq-js-client",
  "version": "1.0.0",
  "type": "module",
  "dependencies": {
    "mqtt": "^5.0.0",
    "express": "^4.18.0",
    "cors": "^2.8.5",
    "ws": "^8.13.0"
  },
  "scripts": {
    "start": "node src/mqtt-client.js",
    "dev": "node --watch src/mqtt-client.js"
  }
}
```

## Summary

MQTT.js is the most mature and feature-complete MQTT client library in the JavaScript ecosystem, with full support for MQTT 5.0 protocol. This library can be used in both Node.js server environments and browsers via WebSocket connections.

Through the examples in this document, you can quickly get started with connecting to RobustMQ MQTT Broker in various JavaScript environments, from simple message sending and receiving to complex real-time web applications. Combined with modern frontend frameworks (React, Vue, etc.), you can build feature-rich real-time messaging applications.

