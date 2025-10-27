# Connecting to RobustMQ with Go SDK

## Overview

Eclipse Paho MQTT Go Client is the Go language client library under the Eclipse Paho project. This library can connect to RobustMQ MQTT Broker to publish messages, subscribe to topics, and receive published messages, supporting fully asynchronous operation modes.

The client depends on Google's proxy and websockets packages, featuring lightweight and high-performance characteristics that are well-suited to Go's concurrent programming model.

## Installing Dependencies

### Install via go get

```bash
go get github.com/eclipse/paho.mqtt.golang
```

### Add dependency in go.mod

```go
module your-project

go 1.19

require (
    github.com/eclipse/paho.mqtt.golang v1.4.3
)
```

## Basic Connection Example

### Publishing and Subscribing Example

```go
package main

import (
	"fmt"
	"log"
	"os"
	"time"

	"github.com/eclipse/paho.mqtt.golang"
)

// Message handler callback function
var messageHandler mqtt.MessageHandler = func(client mqtt.Client, msg mqtt.Message) {
	fmt.Printf("Message received - Topic: %s\n", msg.Topic())
	fmt.Printf("Message content: %s\n", msg.Payload())
	fmt.Printf("QoS: %d\n", msg.Qos())
	fmt.Printf("Retained: %t\n", msg.Retained())
	fmt.Println("==================")
}

// Connection lost callback function
var connectLostHandler mqtt.ConnectionLostHandler = func(client mqtt.Client, err error) {
	fmt.Printf("Connection lost: %v\n", err)
	fmt.Println("Attempting to reconnect...")
}

// Connection success callback function
var connectHandler mqtt.OnConnectHandler = func(client mqtt.Client) {
	fmt.Println("Successfully connected to RobustMQ")
}

func main() {
	// Enable debug logging
	mqtt.DEBUG = log.New(os.Stdout, "[DEBUG] ", 0)
	mqtt.ERROR = log.New(os.Stdout, "[ERROR] ", 0)

	// Create client options
	opts := mqtt.NewClientOptions()
	opts.AddBroker("tcp://localhost:1883")
	opts.SetClientID("robustmq_go_client")
	opts.SetUsername("your_username")
	opts.SetPassword("your_password")
	
	// Set connection parameters
	opts.SetKeepAlive(60 * time.Second)
	opts.SetDefaultPublishHandler(messageHandler)
	opts.SetPingTimeout(1 * time.Second)
	opts.SetConnectTimeout(5 * time.Second)
	opts.SetAutoReconnect(true)
	opts.SetMaxReconnectInterval(1 * time.Minute)
	
	// Set callback functions
	opts.SetConnectionLostHandler(connectLostHandler)
	opts.SetOnConnectHandler(connectHandler)

	// Create client
	client := mqtt.NewClient(opts)
	
	// Connect to RobustMQ Broker
	if token := client.Connect(); token.Wait() && token.Error() != nil {
		panic(token.Error())
	}

	// Subscribe to topic
	topic := "robustmq/go/test/#"
	if token := client.Subscribe(topic, 1, nil); token.Wait() && token.Error() != nil {
		fmt.Println("Subscription failed:", token.Error())
		os.Exit(1)
	}
	fmt.Printf("Successfully subscribed to topic: %s\n", topic)
	
	// Publish message
	pubTopic := "robustmq/go/test/hello"
	payload := "Hello RobustMQ from Go client!"
	token := client.Publish(pubTopic, 1, false, payload)
	token.Wait()
	fmt.Printf("Message published to topic: %s\n", pubTopic)

	// Wait to receive messages
	time.Sleep(3 * time.Second)

	// Unsubscribe
	if token := client.Unsubscribe(topic); token.Wait() && token.Error() != nil {
		fmt.Println("Unsubscribe failed:", token.Error())
		os.Exit(1)
	}
	fmt.Printf("Unsubscribed from topic: %s\n", topic)
  
	// Disconnect
	client.Disconnect(250)
	fmt.Println("Disconnected from RobustMQ")
}
```

## Advanced Features

### SSL/TLS Connection

```go
package main

import (
	"crypto/tls"
	"fmt"
	"time"

	"github.com/eclipse/paho.mqtt.golang"
)

func main() {
	// Create TLS configuration
	tlsConfig := &tls.Config{
		InsecureSkipVerify: false, // Set to false in production
		ClientAuth:         tls.NoClientCert,
		ClientCAs:          nil,
		ServerName:         "localhost", // Server name
	}

	opts := mqtt.NewClientOptions()
	opts.AddBroker("ssl://localhost:1885") // SSL port
	opts.SetClientID("robustmq_go_ssl_client")
	opts.SetTLSConfig(tlsConfig)
	
	// Set connection parameters
	opts.SetKeepAlive(30 * time.Second)
	opts.SetPingTimeout(5 * time.Second)
	opts.SetConnectTimeout(5 * time.Second)
	opts.SetAutoReconnect(true)

	client := mqtt.NewClient(opts)
	
	if token := client.Connect(); token.Wait() && token.Error() != nil {
		panic(token.Error())
	}
	
	fmt.Println("Successfully established SSL connection to RobustMQ")

	// Publish test message
	token := client.Publish("robustmq/ssl/test", 1, false, "SSL connection test")
	token.Wait()
	fmt.Println("SSL message published successfully")

	time.Sleep(1 * time.Second)
	client.Disconnect(250)
}
```

### WebSocket Connection

```go
package main

import (
	"fmt"
	"time"

	"github.com/eclipse/paho.mqtt.golang"
)

func main() {
	opts := mqtt.NewClientOptions()
	opts.AddBroker("ws://localhost:8083/mqtt") // WebSocket port
	opts.SetClientID("robustmq_go_ws_client")
	
	opts.SetKeepAlive(30 * time.Second)
	opts.SetPingTimeout(5 * time.Second)
	opts.SetConnectTimeout(5 * time.Second)

	client := mqtt.NewClient(opts)
	
	if token := client.Connect(); token.Wait() && token.Error() != nil {
		panic(token.Error())
	}
	
	fmt.Println("Successfully established WebSocket connection to RobustMQ")

	// Publish test message
	token := client.Publish("robustmq/ws/test", 1, false, "WebSocket connection test")
	token.Wait()
	fmt.Println("WebSocket message published successfully")

	time.Sleep(1 * time.Second)
	client.Disconnect(250)
}
```

## Performance Optimization

### Connection Pooling

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
	
	// Basic client options
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
			fmt.Printf("Pool client %d connection failed: %v\n", i, token.Error())
			continue
		}
		
		p.clients <- client
	}
	fmt.Printf("Connection pool initialized with %d connections\n", len(p.clients))
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
```

## Connection Parameters

### Basic Connection Parameters

| Parameter | Description | Default Value |
|-----------|-------------|---------------|
| `Broker` | RobustMQ Broker address | tcp://localhost:1883 |
| `ClientID` | Unique client identifier | Auto-generated |
| `KeepAlive` | Heartbeat interval | 30 seconds |
| `ConnectTimeout` | Connection timeout | 30 seconds |
| `AutoReconnect` | Auto reconnection | false |

### RobustMQ Supported Protocol Ports

| Protocol | Port | Connection String Example |
|----------|------|---------------------------|
| MQTT | 1883 | `tcp://localhost:1883` |
| MQTT over SSL | 1885 | `ssl://localhost:1885` |
| MQTT over WebSocket | 8083 | `ws://localhost:8083/mqtt` |
| MQTT over WSS | 8085 | `wss://localhost:8085/mqtt` |

## Best Practices

### 1. Graceful Shutdown

```go
func gracefulShutdown(client mqtt.Client) {
	// Listen for system signals
	c := make(chan os.Signal, 1)
	signal.Notify(c, os.Interrupt, syscall.SIGTERM)
	
	go func() {
		<-c
		fmt.Println("Received shutdown signal, gracefully shutting down...")
		
		// Unsubscribe all topics
		if token := client.Unsubscribe("robustmq/+"); token.Wait() && token.Error() != nil {
			fmt.Printf("Unsubscribe failed: %v\n", token.Error())
		}
		
		// Disconnect
		client.Disconnect(250)
		fmt.Println("Safely closed MQTT connection")
		os.Exit(0)
	}()
}
```

### 2. Error Retry Mechanism

```go
func publishWithRetry(client mqtt.Client, topic, payload string, qos byte, maxRetries int) error {
	for i := 0; i < maxRetries; i++ {
		token := client.Publish(topic, qos, false, payload)
		if token.Wait() && token.Error() != nil {
			fmt.Printf("Publish attempt %d failed: %v\n", i+1, token.Error())
			if i < maxRetries-1 {
				time.Sleep(time.Duration(i+1) * time.Second) // Exponential backoff
			}
		} else {
			fmt.Printf("Message published successfully (attempt %d)\n", i+1)
			return nil
		}
	}
	return fmt.Errorf("publish failed after %d retries", maxRetries)
}
```

### 3. Performance Monitoring

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
	
	fmt.Println("=== MQTT Performance Statistics ===")
	fmt.Printf("Runtime: %v\n", elapsed)
	fmt.Printf("Messages published: %d\n", published)
	fmt.Printf("Messages received: %d\n", received)
	fmt.Printf("Publish errors: %d\n", errors)
	
	if elapsed.Seconds() > 0 {
		fmt.Printf("Publish rate: %.2f msg/s\n", float64(published)/elapsed.Seconds())
		fmt.Printf("Receive rate: %.2f msg/s\n", float64(received)/elapsed.Seconds())
	}
	fmt.Println("===================================")
}
```

## Frequently Asked Questions

### Q: How to handle connection disconnections?

A: Use auto-reconnection and connection lost callbacks:

```go
opts.SetAutoReconnect(true)
opts.SetMaxReconnectInterval(1 * time.Minute)
opts.SetConnectionLostHandler(func(client mqtt.Client, err error) {
    fmt.Printf("Connection lost, will auto-reconnect: %v\n", err)
})
```

### Q: How to set Quality of Service (QoS) levels?

A: RobustMQ supports the three standard MQTT QoS levels:

```go
// QoS 0: At most once delivery
client.Publish(topic, 0, false, "QoS 0 message")

// QoS 1: At least once delivery
client.Publish(topic, 1, false, "QoS 1 message")

// QoS 2: Exactly once delivery
client.Publish(topic, 2, false, "QoS 2 message")
```

### Q: How to handle high concurrent connections?

A: Use Go's concurrency features and connection pools:

1. Utilize goroutines for concurrent operations
2. Implement connection pools to manage multiple MQTT connections
3. Configure connection parameters properly to avoid resource waste

## Compilation and Execution

### Project Structure

```
your-project/
├── main.go
├── go.mod
├── go.sum
└── config/
    └── mqtt.json
```

### Compilation and Execution

```bash
# Initialize Go module
go mod init robustmq-go-client

# Install dependencies
go mod tidy

# Run program
go run main.go

# Build binary
go build -o robustmq-client main.go

# Run compiled program
./robustmq-client
```

## MQTT 5.0 Support

Currently, MQTT 5.0 support in Paho Go client is still under development. It's recommended to follow official updates. For scenarios requiring MQTT 5.0 features, consider using other Go MQTT client libraries such as:

- **gmqtt**: High-performance Go client supporting MQTT 5.0
- **paho.golang**: Next-generation Go client from Paho project (under development)

## Summary

Eclipse Paho MQTT Go Client is a mature and stable MQTT client library in the Go ecosystem. This library fully utilizes Go's concurrency features and provides a clean, easy-to-use API, making it ideal for building high-performance MQTT applications.

Through the examples in this document, you can quickly get started with using Go to connect to RobustMQ MQTT Broker and implement various complex message processing scenarios. Combined with Go's concurrency advantages and RobustMQ's high-performance characteristics, you can build efficient and reliable message delivery systems.

