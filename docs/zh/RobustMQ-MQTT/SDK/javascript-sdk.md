# 使用 JavaScript SDK 连接 RobustMQ

## 概述

MQTT.js 是 JavaScript 语言的 MQTT 客户端库，可用于 Node.js 和浏览器环境连接 RobustMQ MQTT Broker。该库是目前 JavaScript 生态中最受欢迎的 MQTT 客户端库，支持多种连接方式和完整的 MQTT 5.0 协议。

## 安装依赖

### 使用 npm 安装

```bash
npm install mqtt
```

### 使用 yarn 安装

```bash
yarn add mqtt
```

### 在浏览器中使用

```html
<script src="https://unpkg.com/mqtt/dist/mqtt.min.js"></script>
```

## Node.js 环境示例

### 基础连接和发布订阅

```javascript
// 使用 ES6 模块
import mqtt from 'mqtt'
// 或者使用 CommonJS
// const mqtt = require('mqtt')

// 连接选项
const options = {
    clean: true, // true: 清除会话, false: 保留会话
    connectTimeout: 4000, // 超时时间
    // 认证信息
    clientId: 'robustmq_js_client',
    username: 'your_username',
    password: 'your_password',
    keepalive: 60,
    reconnectPeriod: 1000, // 重连间隔
}

// 连接字符串，通过协议指定使用的连接方式
// ws: 未加密 WebSocket 连接
// wss: 加密 WebSocket 连接
// mqtt: 未加密 TCP 连接
// mqtts: 加密 TCP 连接
const connectUrl = 'ws://localhost:8083/mqtt'
const client = mqtt.connect(connectUrl, options)

// 连接成功事件
client.on('connect', () => {
    console.log('成功连接到 RobustMQ')

    // 订阅主题
    const topic = 'robustmq/js/test/#'
    client.subscribe(topic, { qos: 1 }, (err) => {
        if (!err) {
            console.log(`成功订阅主题: ${topic}`)

            // 发布测试消息
            const pubTopic = 'robustmq/js/test/hello'
            const message = 'Hello RobustMQ from JavaScript!'

            client.publish(pubTopic, message, { qos: 1, retain: false }, (err) => {
                if (!err) {
                    console.log(`消息已发布到: ${pubTopic}`)
                } else {
                    console.log('消息发布失败:', err)
                }
            })
        } else {
            console.log('订阅失败:', err)
        }
    })
})

// 重连事件
client.on('reconnect', () => {
    console.log('正在重连到 RobustMQ...')
})

// 错误事件
client.on('error', (error) => {
    console.log('连接错误:', error)
})

// 消息接收事件
client.on('message', (topic, message, packet) => {
    console.log('=== 收到消息 ===')
    console.log('主题:', topic)
    console.log('消息:', message.toString())
    console.log('QoS:', packet.qos)
    console.log('保留消息:', packet.retain)
    console.log('================')
})

// 断开连接事件
client.on('disconnect', () => {
    console.log('已断开与 RobustMQ 的连接')
})

// 优雅关闭
process.on('SIGINT', () => {
    console.log('收到中断信号，正在关闭...')
    client.end()
    process.exit(0)
})
```

## 浏览器环境示例

### HTML 页面示例

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RobustMQ JavaScript 客户端</title>
    <script src="https://unpkg.com/mqtt/dist/mqtt.min.js"></script>
</head>
<body>
    <h1>RobustMQ JavaScript 客户端示例</h1>

    <div>
        <h3>连接状态</h3>
        <p id="status">未连接</p>
        <button id="connectBtn">连接</button>
        <button id="disconnectBtn">断开连接</button>
    </div>

    <div>
        <h3>发布消息</h3>
        <input type="text" id="pubTopic" placeholder="主题" value="robustmq/browser/test">
        <input type="text" id="pubMessage" placeholder="消息内容" value="Hello from browser!">
        <button id="publishBtn">发布</button>
    </div>

    <div>
        <h3>订阅主题</h3>
        <input type="text" id="subTopic" placeholder="订阅主题" value="robustmq/browser/test">
        <button id="subscribeBtn">订阅</button>
        <button id="unsubscribeBtn">取消订阅</button>
    </div>

    <div>
        <h3>接收的消息</h3>
        <div id="messages" style="border: 1px solid #ccc; height: 200px; overflow-y: auto; padding: 10px;"></div>
    </div>

    <script>
        let client = null
        const statusEl = document.getElementById('status')
        const messagesEl = document.getElementById('messages')

        // 连接配置
        const options = {
            clean: true,
            connectTimeout: 4000,
            clientId: 'robustmq_browser_client_' + Math.random().toString(16).substr(2, 8),
            username: 'test_user',
            password: 'test_pass',
        }

        // 连接函数
        function connect() {
            const connectUrl = 'ws://localhost:8083/mqtt'
            client = mqtt.connect(connectUrl, options)

            client.on('connect', () => {
                statusEl.textContent = '已连接到 RobustMQ'
                statusEl.style.color = 'green'
                console.log('浏览器客户端连接成功')
            })

            client.on('error', (err) => {
                statusEl.textContent = '连接错误: ' + err.message
                statusEl.style.color = 'red'
                console.error('连接错误:', err)
            })

            client.on('reconnect', () => {
                statusEl.textContent = '正在重连...'
                statusEl.style.color = 'orange'
            })

            client.on('message', (topic, message, packet) => {
                const msgDiv = document.createElement('div')
                msgDiv.innerHTML = `
                    <strong>主题:</strong> ${topic}<br>
                    <strong>消息:</strong> ${message.toString()}<br>
                    <strong>时间:</strong> ${new Date().toLocaleTimeString()}<br>
                    <hr>
                `
                messagesEl.insertBefore(msgDiv, messagesEl.firstChild)

                // 限制显示的消息数量
                if (messagesEl.children.length > 20) {
                    messagesEl.removeChild(messagesEl.lastChild)
                }
            })

            client.on('disconnect', () => {
                statusEl.textContent = '已断开连接'
                statusEl.style.color = 'gray'
            })
        }

        // 断开连接函数
        function disconnect() {
            if (client) {
                client.end()
                client = null
            }
        }

        // 发布消息函数
        function publishMessage() {
            if (!client || !client.connected) {
                alert('请先连接到 RobustMQ')
                return
            }

            const topic = document.getElementById('pubTopic').value
            const message = document.getElementById('pubMessage').value

            client.publish(topic, message, { qos: 1 }, (err) => {
                if (!err) {
                    console.log('消息发布成功:', topic)
                } else {
                    console.error('消息发布失败:', err)
                }
            })
        }

        // 订阅主题函数
        function subscribeTopic() {
            if (!client || !client.connected) {
                alert('请先连接到 RobustMQ')
                return
            }

            const topic = document.getElementById('subTopic').value

            client.subscribe(topic, { qos: 1 }, (err) => {
                if (!err) {
                    console.log('订阅成功:', topic)
                } else {
                    console.error('订阅失败:', err)
                }
            })
        }

        // 取消订阅函数
        function unsubscribeTopic() {
            if (!client || !client.connected) {
                alert('请先连接到 RobustMQ')
                return
            }

            const topic = document.getElementById('subTopic').value

            client.unsubscribe(topic, (err) => {
                if (!err) {
                    console.log('取消订阅成功:', topic)
                } else {
                    console.error('取消订阅失败:', err)
                }
            })
        }

        // 绑定事件
        document.getElementById('connectBtn').addEventListener('click', connect)
        document.getElementById('disconnectBtn').addEventListener('click', disconnect)
        document.getElementById('publishBtn').addEventListener('click', publishMessage)
        document.getElementById('subscribeBtn').addEventListener('click', subscribeTopic)
        document.getElementById('unsubscribeBtn').addEventListener('click', unsubscribeTopic)
    </script>
</body>
</html>
```

## 高级功能

### React 应用集成

```jsx
import React, { useState, useEffect, useCallback } from 'react'
import mqtt from 'mqtt'

const MQTTComponent = () => {
    const [client, setClient] = useState(null)
    const [isConnected, setIsConnected] = useState(false)
    const [messages, setMessages] = useState([])
    const [connectionStatus, setConnectionStatus] = useState('未连接')

    // 连接到 RobustMQ
    const connectToRobustMQ = useCallback(() => {
        const options = {
            clean: true,
            connectTimeout: 4000,
            clientId: 'robustmq_react_client_' + Math.random().toString(16).substr(2, 8),
            username: 'react_user',
            password: 'react_pass',
        }

        const connectUrl = 'ws://localhost:8083/mqtt'
        const mqttClient = mqtt.connect(connectUrl, options)

        mqttClient.on('connect', () => {
            setIsConnected(true)
            setConnectionStatus('已连接')
            console.log('React 应用连接到 RobustMQ 成功')

            // 自动订阅主题
            mqttClient.subscribe('robustmq/react/+', { qos: 1 })
        })

        mqttClient.on('error', (err) => {
            setConnectionStatus('连接错误: ' + err.message)
            console.error('MQTT 连接错误:', err)
        })

        mqttClient.on('message', (topic, message) => {
            const newMessage = {
                id: Date.now(),
                topic,
                message: message.toString(),
                timestamp: new Date().toLocaleTimeString()
            }

            setMessages(prevMessages => [newMessage, ...prevMessages.slice(0, 49)])
        })

        mqttClient.on('disconnect', () => {
            setIsConnected(false)
            setConnectionStatus('已断开连接')
        })

        setClient(mqttClient)
    }, [])

    // 断开连接
    const disconnect = useCallback(() => {
        if (client) {
            client.end()
            setClient(null)
            setIsConnected(false)
            setConnectionStatus('未连接')
        }
    }, [client])

    // 发布消息
    const publishMessage = useCallback((topic, message) => {
        if (client && isConnected) {
            client.publish(topic, message, { qos: 1 }, (err) => {
                if (!err) {
                    console.log('消息发布成功')
                } else {
                    console.error('消息发布失败:', err)
                }
            })
        }
    }, [client, isConnected])

    // 组件卸载时断开连接
    useEffect(() => {
        return () => {
            if (client) {
                client.end()
            }
        }
    }, [client])

    return (
        <div>
            <h2>RobustMQ React 客户端</h2>

            <div>
                <p>连接状态: {connectionStatus}</p>
                <button onClick={connectToRobustMQ} disabled={isConnected}>
                    连接
                </button>
                <button onClick={disconnect} disabled={!isConnected}>
                    断开连接
                </button>
            </div>

            <div>
                <h3>发布消息</h3>
                <button
                    onClick={() => publishMessage('robustmq/react/hello', 'Hello from React!')}
                    disabled={!isConnected}
                >
                    发送测试消息
                </button>
            </div>

            <div>
                <h3>接收的消息</h3>
                <div style={{ height: '300px', overflowY: 'auto', border: '1px solid #ccc', padding: '10px' }}>
                    {messages.map(msg => (
                        <div key={msg.id} style={{ marginBottom: '10px', borderBottom: '1px solid #eee' }}>
                            <strong>主题:</strong> {msg.topic}<br/>
                            <strong>消息:</strong> {msg.message}<br/>
                            <strong>时间:</strong> {msg.timestamp}
                        </div>
                    ))}
                </div>
            </div>
        </div>
    )
}

export default MQTTComponent
```

### Vue.js 应用集成

```vue
<template>
  <div class="mqtt-client">
    <h2>RobustMQ Vue 客户端</h2>

    <div class="connection-panel">
      <p>连接状态: {{ connectionStatus }}</p>
      <button @click="connect" :disabled="isConnected">连接</button>
      <button @click="disconnect" :disabled="!isConnected">断开连接</button>
    </div>

    <div class="publish-panel">
      <h3>发布消息</h3>
      <input v-model="publishTopic" placeholder="主题" />
      <input v-model="publishMessage" placeholder="消息内容" />
      <button @click="publish" :disabled="!isConnected">发布</button>
    </div>

    <div class="subscribe-panel">
      <h3>订阅管理</h3>
      <input v-model="subscribeTopic" placeholder="订阅主题" />
      <button @click="subscribe" :disabled="!isConnected">订阅</button>
      <button @click="unsubscribe" :disabled="!isConnected">取消订阅</button>
    </div>

    <div class="messages-panel">
      <h3>接收的消息</h3>
      <div class="messages-list">
        <div v-for="msg in messages" :key="msg.id" class="message-item">
          <strong>主题:</strong> {{ msg.topic }}<br>
          <strong>消息:</strong> {{ msg.message }}<br>
          <strong>时间:</strong> {{ msg.timestamp }}
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import mqtt from 'mqtt'

export default {
  name: 'MQTTClient',
  data() {
    return {
      client: null,
      isConnected: false,
      connectionStatus: '未连接',
      messages: [],
      publishTopic: 'robustmq/vue/test',
      publishMessage: 'Hello from Vue!',
      subscribeTopic: 'robustmq/vue/+'
    }
  },
  methods: {
    connect() {
      const options = {
        clean: true,
        connectTimeout: 4000,
        clientId: 'robustmq_vue_client_' + Math.random().toString(16).substr(2, 8),
        username: 'vue_user',
        password: 'vue_pass',
      }

      const connectUrl = 'ws://localhost:8083/mqtt'
      this.client = mqtt.connect(connectUrl, options)

      this.client.on('connect', () => {
        this.isConnected = true
        this.connectionStatus = '已连接'
        console.log('Vue 应用连接到 RobustMQ 成功')
      })

      this.client.on('error', (err) => {
        this.connectionStatus = '连接错误: ' + err.message
        console.error('MQTT 连接错误:', err)
      })

      this.client.on('message', (topic, message) => {
        const newMessage = {
          id: Date.now(),
          topic,
          message: message.toString(),
          timestamp: new Date().toLocaleTimeString()
        }

        this.messages.unshift(newMessage)
        // 限制消息数量
        if (this.messages.length > 50) {
          this.messages = this.messages.slice(0, 50)
        }
      })

      this.client.on('disconnect', () => {
        this.isConnected = false
        this.connectionStatus = '已断开连接'
      })
    },

    disconnect() {
      if (this.client) {
        this.client.end()
        this.client = null
        this.isConnected = false
        this.connectionStatus = '未连接'
      }
    },

    publish() {
      if (this.client && this.isConnected) {
        this.client.publish(this.publishTopic, this.publishMessage, { qos: 1 }, (err) => {
          if (!err) {
            console.log('Vue 消息发布成功')
          } else {
            console.error('Vue 消息发布失败:', err)
          }
        })
      }
    },

    subscribe() {
      if (this.client && this.isConnected) {
        this.client.subscribe(this.subscribeTopic, { qos: 1 }, (err) => {
          if (!err) {
            console.log('Vue 订阅成功:', this.subscribeTopic)
          } else {
            console.error('Vue 订阅失败:', err)
          }
        })
      }
    },

    unsubscribe() {
      if (this.client && this.isConnected) {
        this.client.unsubscribe(this.subscribeTopic, (err) => {
          if (!err) {
            console.log('Vue 取消订阅成功:', this.subscribeTopic)
          } else {
            console.error('Vue 取消订阅失败:', err)
          }
        })
      }
    }
  },

  beforeUnmount() {
    if (this.client) {
      this.client.end()
    }
  }
}
</script>

<style scoped>
.mqtt-client {
  padding: 20px;
  max-width: 800px;
  margin: 0 auto;
}

.connection-panel, .publish-panel, .subscribe-panel, .messages-panel {
  margin-bottom: 20px;
  padding: 15px;
  border: 1px solid #ddd;
  border-radius: 5px;
}

.messages-list {
  height: 300px;
  overflow-y: auto;
  border: 1px solid #ccc;
  padding: 10px;
}

.message-item {
  margin-bottom: 10px;
  padding: 5px;
  background-color: #f5f5f5;
  border-radius: 3px;
}

button {
  margin: 5px;
  padding: 8px 15px;
  border: none;
  border-radius: 3px;
  background-color: #007bff;
  color: white;
  cursor: pointer;
}

button:disabled {
  background-color: #6c757d;
  cursor: not-allowed;
}

input {
  margin: 5px;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 3px;
  width: 200px;
}
</style>
```

### Node.js 服务器示例

```javascript
import mqtt from 'mqtt'
import express from 'express'
import { WebSocketServer } from 'ws'
import cors from 'cors'

class MQTTBridge {
    constructor() {
        this.mqttClient = null
        this.webSocketServer = null
        this.expressApp = express()
        this.clients = new Set()

        this.setupExpress()
        this.connectToRobustMQ()
    }

    setupExpress() {
        this.expressApp.use(cors())
        this.expressApp.use(express.json())

        // REST API 发布消息
        this.expressApp.post('/api/publish', (req, res) => {
            const { topic, message, qos = 1 } = req.body

            if (this.mqttClient && this.mqttClient.connected) {
                this.mqttClient.publish(topic, message, { qos }, (err) => {
                    if (!err) {
                        res.json({ success: true, message: 'Message published' })
                    } else {
                        res.status(500).json({ success: false, error: err.message })
                    }
                })
            } else {
                res.status(503).json({ success: false, error: 'MQTT client not connected' })
            }
        })

        // 获取连接状态
        this.expressApp.get('/api/status', (req, res) => {
            res.json({
                connected: this.mqttClient ? this.mqttClient.connected : false,
                clients: this.clients.size
            })
        })
    }

    connectToRobustMQ() {
        const options = {
            clean: true,
            connectTimeout: 4000,
            clientId: 'robustmq_bridge_server',
            username: 'bridge_user',
            password: 'bridge_pass',
            keepalive: 60,
            reconnectPeriod: 1000,
        }

        const connectUrl = 'mqtt://localhost:1883'
        this.mqttClient = mqtt.connect(connectUrl, options)

        this.mqttClient.on('connect', () => {
            console.log('Bridge 服务器连接到 RobustMQ 成功')

            // 订阅所有消息用于转发
            this.mqttClient.subscribe('robustmq/bridge/+', { qos: 1 })
        })

        this.mqttClient.on('message', (topic, message) => {
            const messageData = {
                topic,
                message: message.toString(),
                timestamp: new Date().toISOString()
            }

            // 转发消息到所有 WebSocket 客户端
            this.broadcastToClients(messageData)
        })

        this.mqttClient.on('error', (err) => {
            console.error('MQTT Bridge 连接错误:', err)
        })
    }

    setupWebSocket(server) {
        this.webSocketServer = new WebSocketServer({ server })

        this.webSocketServer.on('connection', (ws) => {
            this.clients.add(ws)
            console.log('新的 WebSocket 客户端连接')

            ws.on('message', (data) => {
                try {
                    const { action, topic, message } = JSON.parse(data)

                    if (action === 'publish' && this.mqttClient && this.mqttClient.connected) {
                        this.mqttClient.publish(topic, message, { qos: 1 })
                    }
                } catch (err) {
                    console.error('WebSocket 消息处理错误:', err)
                }
            })

            ws.on('close', () => {
                this.clients.delete(ws)
                console.log('WebSocket 客户端断开连接')
            })
        })
    }

    broadcastToClients(message) {
        const data = JSON.stringify(message)

        this.clients.forEach(client => {
            if (client.readyState === client.OPEN) {
                client.send(data)
            }
        })
    }

    start(port = 3000) {
        const server = this.expressApp.listen(port, () => {
            console.log(`MQTT Bridge 服务器启动在端口 ${port}`)
        })

        this.setupWebSocket(server)
        return server
    }
}

// 启动 Bridge 服务器
const bridge = new MQTTBridge()
bridge.start(3000)

// 优雅关闭
process.on('SIGINT', () => {
    console.log('收到中断信号，正在关闭服务器...')
    if (bridge.mqttClient) {
        bridge.mqttClient.end()
    }
    process.exit(0)
})
```

## MQTT 5.0 支持

MQTT.js 已经完整支持 MQTT 5.0 协议：

### MQTT 5.0 连接示例

```javascript
import mqtt from 'mqtt'

// MQTT 5.0 连接选项
const options = {
    protocolVersion: 5, // 指定 MQTT 5.0
    clean: true,
    connectTimeout: 4000,
    clientId: 'robustmq_mqtt5_js_client',
    username: 'mqtt5_user',
    password: 'mqtt5_pass',

    // MQTT 5.0 特有属性
    properties: {
        sessionExpiryInterval: 3600, // 会话过期时间：1小时
        receiveMaximum: 100, // 接收最大值
        maximumPacketSize: 1048576, // 最大包大小：1MB
        requestResponseInformation: true,
        requestProblemInformation: true
    }
}

const client = mqtt.connect('ws://localhost:8083/mqtt', options)

client.on('connect', (connack) => {
    console.log('MQTT 5.0 连接成功')
    console.log('服务器响应:', connack)

    // MQTT 5.0 订阅，带属性
    const subscribeOptions = {
        qos: 1,
        properties: {
            subscriptionIdentifier: 1
        }
    }

    client.subscribe('robustmq/mqtt5/test', subscribeOptions, (err, granted) => {
        if (!err) {
            console.log('MQTT 5.0 订阅成功:', granted)
        }
    })
})

client.on('message', (topic, message, packet) => {
    console.log('MQTT 5.0 消息接收:')
    console.log('主题:', topic)
    console.log('消息:', message.toString())
    console.log('包信息:', packet.properties)
})

// 发布 MQTT 5.0 消息
function publishMQTT5Message() {
    const topic = 'robustmq/mqtt5/test'
    const message = 'MQTT 5.0 message from JavaScript'

    const publishOptions = {
        qos: 1,
        retain: false,
        properties: {
            messageExpiryInterval: 300, // 消息过期时间：5分钟
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
            console.log('MQTT 5.0 消息发布成功')
        } else {
            console.error('MQTT 5.0 消息发布失败:', err)
        }
    })
}

// 5秒后发布测试消息
setTimeout(publishMQTT5Message, 5000)
```

## 连接参数配置

### 基础连接参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `host` | RobustMQ Broker 地址 | localhost |
| `port` | 连接端口 | 1883 |
| `clientId` | 客户端唯一标识 | 自动生成 |
| `clean` | 是否清除会话 | true |
| `keepalive` | 心跳间隔（秒） | 60 |
| `connectTimeout` | 连接超时（毫秒） | 30000 |

### RobustMQ 支持的连接方式

| 协议 | 端口 | 连接字符串示例 |
|------|------|----------------|
| MQTT (TCP) | 1883 | `mqtt://localhost:1883` |
| MQTTS (SSL) | 1885 | `mqtts://localhost:1885` |
| WebSocket | 8083 | `ws://localhost:8083/mqtt` |
| WebSocket SSL | 8085 | `wss://localhost:8085/mqtt` |

## 最佳实践

### 1. 错误处理和重连

```javascript
const client = mqtt.connect('ws://localhost:8083/mqtt', {
    reconnectPeriod: 1000, // 重连间隔
    connectTimeout: 4000,
    clean: true
})

client.on('error', (err) => {
    console.error('MQTT 错误:', err)

    // 根据错误类型进行不同处理
    if (err.code === 'ECONNREFUSED') {
        console.log('服务器拒绝连接，检查服务器状态')
    } else if (err.code === 'ENOTFOUND') {
        console.log('无法解析服务器地址')
    }
})

client.on('offline', () => {
    console.log('客户端离线')
})

client.on('reconnect', () => {
    console.log('正在重连...')
})
```

### 2. 消息缓存和离线处理

```javascript
class OfflineMessageCache {
    constructor() {
        this.cache = []
        this.maxCacheSize = 1000
    }

    addMessage(topic, message, options) {
        if (this.cache.length >= this.maxCacheSize) {
            this.cache.shift() // 移除最旧的消息
        }

        this.cache.push({
            topic,
            message,
            options,
            timestamp: Date.now()
        })
    }

    publishCachedMessages(client) {
        console.log(`发布 ${this.cache.length} 条缓存消息`)

        this.cache.forEach(msg => {
            client.publish(msg.topic, msg.message, msg.options)
        })

        this.cache = []
    }
}

const messageCache = new OfflineMessageCache()

const client = mqtt.connect('ws://localhost:8083/mqtt')

client.on('connect', () => {
    console.log('重新连接，发布缓存消息')
    messageCache.publishCachedMessages(client)
})

client.on('offline', () => {
    console.log('客户端离线，消息将被缓存')
})

// 智能发布函数
function smartPublish(topic, message, options = {}) {
    if (client.connected) {
        client.publish(topic, message, options)
    } else {
        console.log('离线状态，消息已缓存')
        messageCache.addMessage(topic, message, options)
    }
}
```

## 常见问题

### Q: 如何在浏览器中连接 RobustMQ？

A: 浏览器只能使用 WebSocket 连接：

```javascript
// 浏览器中只能使用 WebSocket
const client = mqtt.connect('ws://localhost:8083/mqtt', options)
// 或者加密连接
const client = mqtt.connect('wss://localhost:8085/mqtt', options)
```

### Q: 如何处理大量消息？

A: 使用消息队列和批量处理：

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
    console.log(`批量处理 ${messages.length} 条消息`)
    // 批量处理逻辑
}
```

### Q: 如何设置消息质量等级 (QoS)？

A: 在发布和订阅时指定 QoS：

```javascript
// 发布时设置 QoS
client.publish(topic, message, { qos: 1 })

// 订阅时设置 QoS
client.subscribe(topic, { qos: 2 })
```

## 项目结构和部署

### Node.js 项目结构

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

### package.json 示例

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

## 总结

MQTT.js 是 JavaScript 生态中最成熟和功能完整的 MQTT 客户端库，完整支持 MQTT 5.0 协议。该库既可以在 Node.js 服务器环境中使用，也可以在浏览器中通过 WebSocket 连接使用。

通过本文档的示例，您可以快速上手在各种 JavaScript 环境中连接 RobustMQ MQTT Broker，从简单的消息收发到复杂的实时 Web 应用都能轻松实现。结合现代前端框架（React、Vue等），可以构建出功能丰富的实时消息应用。
