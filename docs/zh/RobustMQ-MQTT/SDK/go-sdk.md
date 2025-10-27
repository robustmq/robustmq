# 使用 Go SDK 连接 RobustMQ

## 概述

Eclipse Paho MQTT Go Client 是 Eclipse Paho 项目下的 Go 语言版客户端库，该库能够连接到 RobustMQ MQTT Broker 以发布消息，订阅主题并接收已发布的消息，支持完全异步的操作模式。

该客户端依赖于 Google 的 proxy 和 websockets 软件包，具有轻量级、高性能的特点，非常适合 Go 语言的并发编程模型。

## 安装依赖

### 通过 go get 安装

```bash
go get github.com/eclipse/paho.mqtt.golang
```

### 在 go.mod 中添加依赖

```go
module your-project

go 1.19

require (
    github.com/eclipse/paho.mqtt.golang v1.4.3
)
```

## 基础连接示例

### 发布和订阅示例

```go
package main

import (
	"fmt"
	"log"
	"os"
	"time"

	"github.com/eclipse/paho.mqtt.golang"
)

// 消息处理回调函数
var messageHandler mqtt.MessageHandler = func(client mqtt.Client, msg mqtt.Message) {
	fmt.Printf("收到消息 - 主题: %s\n", msg.Topic())
	fmt.Printf("消息内容: %s\n", msg.Payload())
	fmt.Printf("QoS: %d\n", msg.Qos())
	fmt.Printf("保留消息: %t\n", msg.Retained())
	fmt.Println("==================")
}

// 连接丢失回调函数
var connectLostHandler mqtt.ConnectionLostHandler = func(client mqtt.Client, err error) {
	fmt.Printf("连接丢失: %v\n", err)
	fmt.Println("尝试重新连接...")
}

// 连接成功回调函数
var connectHandler mqtt.OnConnectHandler = func(client mqtt.Client) {
	fmt.Println("成功连接到 RobustMQ")
}

func main() {
	// 启用调试日志
	mqtt.DEBUG = log.New(os.Stdout, "[DEBUG] ", 0)
	mqtt.ERROR = log.New(os.Stdout, "[ERROR] ", 0)

	// 创建客户端选项
	opts := mqtt.NewClientOptions()
	opts.AddBroker("tcp://localhost:1883")
	opts.SetClientID("robustmq_go_client")
	opts.SetUsername("your_username")
	opts.SetPassword("your_password")
	
	// 设置连接参数
	opts.SetKeepAlive(60 * time.Second)
	opts.SetDefaultPublishHandler(messageHandler)
	opts.SetPingTimeout(1 * time.Second)
	opts.SetConnectTimeout(5 * time.Second)
	opts.SetAutoReconnect(true)
	opts.SetMaxReconnectInterval(1 * time.Minute)
	
	// 设置回调函数
	opts.SetConnectionLostHandler(connectLostHandler)
	opts.SetOnConnectHandler(connectHandler)

	// 创建客户端
	client := mqtt.NewClient(opts)
	
	// 连接到 RobustMQ Broker
	if token := client.Connect(); token.Wait() && token.Error() != nil {
		panic(token.Error())
	}

	// 订阅主题
	topic := "robustmq/go/test/#"
	if token := client.Subscribe(topic, 1, nil); token.Wait() && token.Error() != nil {
		fmt.Println("订阅失败:", token.Error())
		os.Exit(1)
	}
	fmt.Printf("成功订阅主题: %s\n", topic)
	
	// 发布消息
	pubTopic := "robustmq/go/test/hello"
	payload := "Hello RobustMQ from Go client!"
	token := client.Publish(pubTopic, 1, false, payload)
	token.Wait()
	fmt.Printf("消息已发布到主题: %s\n", pubTopic)

	// 等待接收消息
	time.Sleep(3 * time.Second)

	// 取消订阅
	if token := client.Unsubscribe(topic); token.Wait() && token.Error() != nil {
		fmt.Println("取消订阅失败:", token.Error())
		os.Exit(1)
	}
	fmt.Printf("已取消订阅主题: %s\n", topic)
  
	// 断开连接
	client.Disconnect(250)
	fmt.Println("已断开与 RobustMQ 的连接")
}
```

## 高级功能

### SSL/TLS 连接

```go
package main

import (
	"crypto/tls"
	"fmt"
	"time"

	"github.com/eclipse/paho.mqtt.golang"
)

func main() {
	// 创建 TLS 配置
	tlsConfig := &tls.Config{
		InsecureSkipVerify: false, // 生产环境设置为 false
		ClientAuth:         tls.NoClientCert,
		ClientCAs:          nil,
		ServerName:         "localhost", // 服务器名称
	}

	opts := mqtt.NewClientOptions()
	opts.AddBroker("ssl://localhost:1885") // SSL 端口
	opts.SetClientID("robustmq_go_ssl_client")
	opts.SetTLSConfig(tlsConfig)
	
	// 设置连接参数
	opts.SetKeepAlive(30 * time.Second)
	opts.SetPingTimeout(5 * time.Second)
	opts.SetConnectTimeout(5 * time.Second)
	opts.SetAutoReconnect(true)

	client := mqtt.NewClient(opts)
	
	if token := client.Connect(); token.Wait() && token.Error() != nil {
		panic(token.Error())
	}
	
	fmt.Println("成功建立 SSL 连接到 RobustMQ")

	// 发布测试消息
	token := client.Publish("robustmq/ssl/test", 1, false, "SSL connection test")
	token.Wait()
	fmt.Println("SSL 消息发布成功")

	time.Sleep(1 * time.Second)
	client.Disconnect(250)
}
```

### WebSocket 连接

```go
package main

import (
	"fmt"
	"time"

	"github.com/eclipse/paho.mqtt.golang"
)

func main() {
	opts := mqtt.NewClientOptions()
	opts.AddBroker("ws://localhost:8083/mqtt") // WebSocket 端口
	opts.SetClientID("robustmq_go_ws_client")
	
	opts.SetKeepAlive(30 * time.Second)
	opts.SetPingTimeout(5 * time.Second)
	opts.SetConnectTimeout(5 * time.Second)

	client := mqtt.NewClient(opts)
	
	if token := client.Connect(); token.Wait() && token.Error() != nil {
		panic(token.Error())
	}
	
	fmt.Println("成功建立 WebSocket 连接到 RobustMQ")

	// 发布测试消息
	token := client.Publish("robustmq/ws/test", 1, false, "WebSocket connection test")
	token.Wait()
	fmt.Println("WebSocket 消息发布成功")

	time.Sleep(1 * time.Second)
	client.Disconnect(250)
}
```

### 并发发布示例

```go
package main

import (
	"fmt"
	"sync"
	"time"

	"github.com/eclipse/paho.mqtt.golang"
)

func main() {
	opts := mqtt.NewClientOptions()
	opts.AddBroker("tcp://localhost:1883")
	opts.SetClientID("robustmq_go_concurrent_client")
	opts.SetKeepAlive(30 * time.Second)
	opts.SetAutoReconnect(true)

	client := mqtt.NewClient(opts)
	
	if token := client.Connect(); token.Wait() && token.Error() != nil {
		panic(token.Error())
	}
	
	fmt.Println("连接成功，开始并发发布消息...")

	var wg sync.WaitGroup
	messageCount := 100
	
	// 并发发布消息
	for i := 0; i < messageCount; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			
			topic := fmt.Sprintf("robustmq/concurrent/test/%d", id)
			payload := fmt.Sprintf("并发消息 #%d", id)
			
			token := client.Publish(topic, 1, false, payload)
			if token.Wait() && token.Error() != nil {
				fmt.Printf("发布消息 %d 失败: %v\n", id, token.Error())
			} else {
				fmt.Printf("消息 %d 发布成功\n", id)
			}
		}(i)
	}
	
	// 等待所有消息发布完成
	wg.Wait()
	fmt.Printf("所有 %d 条消息发布完成\n", messageCount)

	client.Disconnect(250)
}
```

### 高级订阅处理

```go
package main

import (
	"fmt"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/eclipse/paho.mqtt.golang"
)

// 不同主题的消息处理器
func deviceDataHandler(client mqtt.Client, msg mqtt.Message) {
	fmt.Printf("[设备数据] 主题: %s, 消息: %s\n", msg.Topic(), string(msg.Payload()))
}

func systemEventHandler(client mqtt.Client, msg mqtt.Message) {
	fmt.Printf("[系统事件] 主题: %s, 消息: %s\n", msg.Topic(), string(msg.Payload()))
}

func alertHandler(client mqtt.Client, msg mqtt.Message) {
	fmt.Printf("[告警信息] 主题: %s, 消息: %s\n", msg.Topic(), string(msg.Payload()))
}

func main() {
	opts := mqtt.NewClientOptions()
	opts.AddBroker("tcp://localhost:1883")
	opts.SetClientID("robustmq_go_advanced_subscriber")
	opts.SetKeepAlive(30 * time.Second)
	opts.SetAutoReconnect(true)
	
	// 连接成功回调
	opts.SetOnConnectHandler(func(client mqtt.Client) {
		fmt.Println("连接成功，开始订阅多个主题...")
		
		// 订阅不同类型的主题，使用不同的处理器
		subscriptions := map[string]mqtt.MessageHandler{
			"robustmq/device/+/data":   deviceDataHandler,
			"robustmq/system/events":   systemEventHandler,
			"robustmq/alerts/+":        alertHandler,
		}
		
		for topic, handler := range subscriptions {
			if token := client.Subscribe(topic, 1, handler); token.Wait() && token.Error() != nil {
				fmt.Printf("订阅 %s 失败: %v\n", topic, token.Error())
			} else {
				fmt.Printf("成功订阅: %s\n", topic)
			}
		}
	})

	client := mqtt.NewClient(opts)
	
	if token := client.Connect(); token.Wait() && token.Error() != nil {
		panic(token.Error())
	}

	// 模拟发布一些测试消息
	go func() {
		time.Sleep(2 * time.Second)
		
		testMessages := map[string]string{
			"robustmq/device/sensor1/data": `{"temperature": 25.5, "humidity": 60}`,
			"robustmq/system/events":       `{"event": "system_startup", "timestamp": "2024-01-01T10:00:00Z"}`,
			"robustmq/alerts/cpu":          `{"level": "warning", "cpu_usage": 85}`,
		}
		
		for topic, payload := range testMessages {
			token := client.Publish(topic, 1, false, payload)
			token.Wait()
			fmt.Printf("测试消息已发布到: %s\n", topic)
			time.Sleep(500 * time.Millisecond)
		}
	}()

	// 等待中断信号
	c := make(chan os.Signal, 1)
	signal.Notify(c, os.Interrupt, syscall.SIGTERM)
	
	fmt.Println("客户端运行中，按 Ctrl+C 退出...")
	<-c
	
	fmt.Println("正在断开连接...")
	client.Disconnect(250)
	fmt.Println("已断开与 RobustMQ 的连接")
}
```

## 高级功能

### 连接池实现

```go
package main

import (
	"fmt"
	"sync"
	"time"

	"github.com/eclipse/paho.mqtt.golang"
)

type MQTTConnectionPool struct {
	broker     string
	poolSize   int
	clients    chan mqtt.Client
	clientOpts *mqtt.ClientOptions
	mu         sync.RWMutex
}

func NewMQTTConnectionPool(broker string, poolSize int) *MQTTConnectionPool {
	pool := &MQTTConnectionPool{
		broker:   broker,
		poolSize: poolSize,
		clients:  make(chan mqtt.Client, poolSize),
	}
	
	// 基础客户端选项
	pool.clientOpts = mqtt.NewClientOptions()
	pool.clientOpts.AddBroker(broker)
	pool.clientOpts.SetKeepAlive(30 * time.Second)
	pool.clientOpts.SetAutoReconnect(true)
	pool.clientOpts.SetMaxReconnectInterval(1 * time.Minute)
	
	pool.initializePool()
	return pool
}

func (p *MQTTConnectionPool) initializePool() {
	for i := 0; i < p.poolSize; i++ {
		clientID := fmt.Sprintf("robustmq_pool_client_%d", i)
		opts := p.clientOpts
		opts.SetClientID(clientID)
		
		client := mqtt.NewClient(opts)
		if token := client.Connect(); token.Wait() && token.Error() != nil {
			fmt.Printf("连接池客户端 %d 连接失败: %v\n", i, token.Error())
			continue
		}
		
		p.clients <- client
	}
	fmt.Printf("连接池初始化完成，共 %d 个连接\n", len(p.clients))
}

func (p *MQTTConnectionPool) GetClient() mqtt.Client {
	return <-p.clients
}

func (p *MQTTConnectionPool) ReturnClient(client mqtt.Client) {
	if client.IsConnected() {
		p.clients <- client
	}
}

func (p *MQTTConnectionPool) Close() {
	close(p.clients)
	for client := range p.clients {
		client.Disconnect(250)
	}
}

// 使用连接池的示例
func main() {
	pool := NewMQTTConnectionPool("tcp://localhost:1883", 5)
	defer pool.Close()
	
	// 使用连接池发布消息
	for i := 0; i < 20; i++ {
		client := pool.GetClient()
		
		topic := fmt.Sprintf("robustmq/pool/test/%d", i)
		payload := fmt.Sprintf("连接池消息 #%d", i)
		
		token := client.Publish(topic, 1, false, payload)
		token.Wait()
		
		fmt.Printf("消息 %d 发布成功\n", i)
		pool.ReturnClient(client)
		
		time.Sleep(100 * time.Millisecond)
	}
	
	fmt.Println("所有消息发布完成")
}
```

### 结构化消息处理

```go
package main

import (
	"encoding/json"
	"fmt"
	"time"

	"github.com/eclipse/paho.mqtt.golang"
)

// 设备数据结构
type DeviceData struct {
	DeviceID    string    `json:"device_id"`
	Temperature float64   `json:"temperature"`
	Humidity    float64   `json:"humidity"`
	Timestamp   time.Time `json:"timestamp"`
}

// 系统事件结构
type SystemEvent struct {
	EventType string                 `json:"event_type"`
	Data      map[string]interface{} `json:"data"`
	Timestamp time.Time              `json:"timestamp"`
}

// 设备数据处理器
func handleDeviceData(client mqtt.Client, msg mqtt.Message) {
	var data DeviceData
	if err := json.Unmarshal(msg.Payload(), &data); err != nil {
		fmt.Printf("解析设备数据失败: %v\n", err)
		return
	}
	
	fmt.Printf("设备 %s 数据: 温度=%.1f°C, 湿度=%.1f%%\n", 
		data.DeviceID, data.Temperature, data.Humidity)
	
	// 处理设备数据逻辑
	if data.Temperature > 30 {
		// 发送高温告警
		alert := SystemEvent{
			EventType: "high_temperature_alert",
			Data: map[string]interface{}{
				"device_id":   data.DeviceID,
				"temperature": data.Temperature,
			},
			Timestamp: time.Now(),
		}
		
		alertJSON, _ := json.Marshal(alert)
		client.Publish("robustmq/alerts/temperature", 1, false, alertJSON)
	}
}

// 系统事件处理器
func handleSystemEvent(client mqtt.Client, msg mqtt.Message) {
	var event SystemEvent
	if err := json.Unmarshal(msg.Payload(), &event); err != nil {
		fmt.Printf("解析系统事件失败: %v\n", err)
		return
	}
	
	fmt.Printf("系统事件: %s, 数据: %+v\n", event.EventType, event.Data)
}

func main() {
	opts := mqtt.NewClientOptions()
	opts.AddBroker("tcp://localhost:1883")
	opts.SetClientID("robustmq_go_structured_client")
	opts.SetKeepAlive(30 * time.Second)
	opts.SetAutoReconnect(true)

	client := mqtt.NewClient(opts)
	
	if token := client.Connect(); token.Wait() && token.Error() != nil {
		panic(token.Error())
	}
	
	fmt.Println("连接成功，开始订阅结构化消息...")

	// 订阅不同类型的消息
	client.Subscribe("robustmq/devices/+/data", 1, handleDeviceData)
	client.Subscribe("robustmq/system/events", 1, handleSystemEvent)
	
	// 模拟发送设备数据
	go func() {
		for i := 0; i < 5; i++ {
			deviceData := DeviceData{
				DeviceID:    fmt.Sprintf("sensor_%d", i+1),
				Temperature: 20.0 + float64(i*5),
				Humidity:    50.0 + float64(i*2),
				Timestamp:   time.Now(),
			}
			
			dataJSON, _ := json.Marshal(deviceData)
			topic := fmt.Sprintf("robustmq/devices/%s/data", deviceData.DeviceID)
			
			client.Publish(topic, 1, false, dataJSON)
			time.Sleep(1 * time.Second)
		}
	}()

	// 运行 10 秒
	time.Sleep(10 * time.Second)
	client.Disconnect(250)
}
```

### 性能监控和指标

```go
package main

import (
	"fmt"
	"sync/atomic"
	"time"

	"github.com/eclipse/paho.mqtt.golang"
)

type MQTTMetrics struct {
	MessagesPublished int64
	MessagesReceived  int64
	PublishErrors     int64
	StartTime         time.Time
}

func (m *MQTTMetrics) IncrementPublished() {
	atomic.AddInt64(&m.MessagesPublished, 1)
}

func (m *MQTTMetrics) IncrementReceived() {
	atomic.AddInt64(&m.MessagesReceived, 1)
}

func (m *MQTTMetrics) IncrementErrors() {
	atomic.AddInt64(&m.PublishErrors, 1)
}

func (m *MQTTMetrics) PrintStats() {
	elapsed := time.Since(m.StartTime)
	published := atomic.LoadInt64(&m.MessagesPublished)
	received := atomic.LoadInt64(&m.MessagesReceived)
	errors := atomic.LoadInt64(&m.PublishErrors)
	
	fmt.Println("=== MQTT 性能统计 ===")
	fmt.Printf("运行时间: %v\n", elapsed)
	fmt.Printf("已发布消息: %d\n", published)
	fmt.Printf("已接收消息: %d\n", received)
	fmt.Printf("发布错误: %d\n", errors)
	
	if elapsed.Seconds() > 0 {
		fmt.Printf("发布速率: %.2f 消息/秒\n", float64(published)/elapsed.Seconds())
		fmt.Printf("接收速率: %.2f 消息/秒\n", float64(received)/elapsed.Seconds())
	}
	fmt.Println("===================")
}

func main() {
	metrics := &MQTTMetrics{StartTime: time.Now()}
	
	opts := mqtt.NewClientOptions()
	opts.AddBroker("tcp://localhost:1883")
	opts.SetClientID("robustmq_go_metrics_client")
	opts.SetKeepAlive(30 * time.Second)
	
	// 消息接收处理器
	opts.SetDefaultPublishHandler(func(client mqtt.Client, msg mqtt.Message) {
		metrics.IncrementReceived()
		fmt.Printf("收到消息: %s -> %s\n", msg.Topic(), string(msg.Payload()))
	})

	client := mqtt.NewClient(opts)
	
	if token := client.Connect(); token.Wait() && token.Error() != nil {
		panic(token.Error())
	}

	// 订阅测试主题
	client.Subscribe("robustmq/metrics/test", 1, nil)
	
	// 定期打印统计信息
	ticker := time.NewTicker(5 * time.Second)
	go func() {
		for range ticker.C {
			metrics.PrintStats()
		}
	}()
	
	// 发布测试消息
	go func() {
		for i := 0; i < 100; i++ {
			payload := fmt.Sprintf("性能测试消息 #%d", i)
			token := client.Publish("robustmq/metrics/test", 1, false, payload)
			
			if token.Wait() && token.Error() != nil {
				metrics.IncrementErrors()
			} else {
				metrics.IncrementPublished()
			}
			
			time.Sleep(100 * time.Millisecond)
		}
	}()

	// 运行 15 秒
	time.Sleep(15 * time.Second)
	ticker.Stop()
	
	metrics.PrintStats()
	client.Disconnect(250)
}
```

## 连接参数配置

### 基础连接参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `Broker` | RobustMQ Broker 地址 | tcp://localhost:1883 |
| `ClientID` | 客户端唯一标识 | 自动生成 |
| `KeepAlive` | 心跳间隔 | 30秒 |
| `ConnectTimeout` | 连接超时时间 | 30秒 |
| `AutoReconnect` | 自动重连 | false |

### RobustMQ 支持的协议端口

| 协议 | 端口 | 连接字符串示例 |
|------|------|---------------|
| MQTT | 1883 | `tcp://localhost:1883` |
| MQTT over SSL | 1885 | `ssl://localhost:1885` |
| MQTT over WebSocket | 8083 | `ws://localhost:8083/mqtt` |
| MQTT over WSS | 8084 | `wss://localhost:8084/mqtt` |

## 最佳实践

### 1. 优雅关闭

```go
func gracefulShutdown(client mqtt.Client) {
	// 监听系统信号
	c := make(chan os.Signal, 1)
	signal.Notify(c, os.Interrupt, syscall.SIGTERM)
	
	go func() {
		<-c
		fmt.Println("收到关闭信号，正在优雅关闭...")
		
		// 取消所有订阅
		if token := client.Unsubscribe("robustmq/+"); token.Wait() && token.Error() != nil {
			fmt.Printf("取消订阅失败: %v\n", token.Error())
		}
		
		// 断开连接
		client.Disconnect(250)
		fmt.Println("已安全关闭 MQTT 连接")
		os.Exit(0)
	}()
}
```

### 2. 错误重试机制

```go
func publishWithRetry(client mqtt.Client, topic, payload string, qos byte, maxRetries int) error {
	for i := 0; i < maxRetries; i++ {
		token := client.Publish(topic, qos, false, payload)
		if token.Wait() && token.Error() != nil {
			fmt.Printf("发布尝试 %d 失败: %v\n", i+1, token.Error())
			if i < maxRetries-1 {
				time.Sleep(time.Duration(i+1) * time.Second) // 指数退避
			}
		} else {
			fmt.Printf("消息发布成功 (尝试 %d 次)\n", i+1)
			return nil
		}
	}
	return fmt.Errorf("发布失败，已重试 %d 次", maxRetries)
}
```

### 3. 配置管理

```go
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"time"

	"github.com/eclipse/paho.mqtt.golang"
)

// MQTT 配置结构
type MQTTConfig struct {
	Broker            string        `json:"broker"`
	ClientID          string        `json:"client_id"`
	Username          string        `json:"username"`
	Password          string        `json:"password"`
	KeepAlive         time.Duration `json:"keep_alive"`
	ConnectTimeout    time.Duration `json:"connect_timeout"`
	AutoReconnect     bool          `json:"auto_reconnect"`
	MaxReconnectDelay time.Duration `json:"max_reconnect_delay"`
	CleanSession      bool          `json:"clean_session"`
}

// 从配置文件加载配置
func LoadMQTTConfig(configFile string) (*MQTTConfig, error) {
	file, err := os.Open(configFile)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	var config MQTTConfig
	decoder := json.NewDecoder(file)
	if err := decoder.Decode(&config); err != nil {
		return nil, err
	}

	return &config, nil
}

// 根据配置创建客户端选项
func (config *MQTTConfig) ToClientOptions() *mqtt.ClientOptions {
	opts := mqtt.NewClientOptions()
	opts.AddBroker(config.Broker)
	opts.SetClientID(config.ClientID)
	opts.SetUsername(config.Username)
	opts.SetPassword(config.Password)
	opts.SetKeepAlive(config.KeepAlive)
	opts.SetConnectTimeout(config.ConnectTimeout)
	opts.SetAutoReconnect(config.AutoReconnect)
	opts.SetMaxReconnectInterval(config.MaxReconnectDelay)
	opts.SetCleanSession(config.CleanSession)
	
	return opts
}

func main() {
	// 示例配置
	config := &MQTTConfig{
		Broker:            "tcp://localhost:1883",
		ClientID:          "robustmq_go_config_client",
		Username:          "test_user",
		Password:          "test_pass",
		KeepAlive:         30 * time.Second,
		ConnectTimeout:    5 * time.Second,
		AutoReconnect:     true,
		MaxReconnectDelay: 1 * time.Minute,
		CleanSession:      true,
	}

	opts := config.ToClientOptions()
	client := mqtt.NewClient(opts)
	
	if token := client.Connect(); token.Wait() && token.Error() != nil {
		panic(token.Error())
	}
	
	fmt.Println("使用配置成功连接到 RobustMQ")
	
	// ... 其他操作 ...
	
	client.Disconnect(250)
}
```

## 常见问题

### Q: 如何处理连接断开？

A: 使用自动重连和连接丢失回调：

```go
opts.SetAutoReconnect(true)
opts.SetMaxReconnectInterval(1 * time.Minute)
opts.SetConnectionLostHandler(func(client mqtt.Client, err error) {
    fmt.Printf("连接丢失，将自动重连: %v\n", err)
})
```

### Q: 如何设置消息质量等级 (QoS)？

A: RobustMQ 支持 MQTT 标准的三种 QoS 等级：

```go
// QoS 0: 最多一次传递
client.Publish(topic, 0, false, "QoS 0 message")

// QoS 1: 至少一次传递
client.Publish(topic, 1, false, "QoS 1 message")

// QoS 2: 恰好一次传递
client.Publish(topic, 2, false, "QoS 2 message")
```

### Q: 如何处理大量并发连接？

A: 使用 Go 的并发特性和连接池：

1. 利用 goroutines 处理并发操作
2. 实现连接池管理多个 MQTT 连接
3. 合理配置连接参数避免资源浪费

## 编译和运行

### 项目结构

```
your-project/
├── main.go
├── go.mod
├── go.sum
└── config/
    └── mqtt.json
```

### 编译运行

```bash
# 初始化 Go 模块
go mod init robustmq-go-client

# 安装依赖
go mod tidy

# 运行程序
go run main.go

# 编译二进制文件
go build -o robustmq-client main.go

# 运行编译后的程序
./robustmq-client
```

## MQTT 5.0 支持

目前 Paho Go 客户端对 MQTT 5.0 的支持还在开发中，建议关注官方更新。对于需要 MQTT 5.0 特性的场景，可以考虑使用其他 Go MQTT 客户端库，如：

- **gmqtt**: 支持 MQTT 5.0 的高性能 Go 客户端
- **paho.golang**: Paho 项目的新一代 Go 客户端（开发中）

## 总结

Eclipse Paho MQTT Go Client 是 Go 语言生态中成熟稳定的 MQTT 客户端库。该库充分利用了 Go 语言的并发特性，提供了简洁易用的 API，非常适合构建高性能的 MQTT 应用程序。

通过本文档的示例，您可以快速上手使用 Go 语言连接 RobustMQ MQTT Broker，并实现各种复杂的消息处理场景。结合 Go 语言的并发优势和 RobustMQ 的高性能特性，可以构建出既高效又可靠的消息传递系统。

