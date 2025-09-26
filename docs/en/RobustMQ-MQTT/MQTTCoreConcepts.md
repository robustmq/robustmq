# MQTT Core Concepts

## Publish-Subscribe

MQTT is based on the publish-subscribe pattern, which decouples message senders (publishers) and receivers (subscribers) by introducing an intermediary broker role to handle message routing and distribution.

Publishers and subscribers don't need to know about each other's existence. The only connection between them is a consistent agreement about messages, such as what topic the message will use, what fields the message will contain, etc. This makes MQTT communication more flexible, as we can dynamically add or remove subscribers and publishers at any time.

Through publish-subscribe, we can easily implement message broadcasting, multicasting, and unicasting.

### Advantages of Publish-Subscribe

- **Decoupling**: Publishers and subscribers are independent and don't need direct communication
- **Scalability**: Can dynamically add or remove publishers and subscribers
- **Flexibility**: Supports one-to-many and many-to-many message delivery patterns
- **Asynchrony**: Publishers and subscribers can work asynchronously

## Server

Acts as an intermediary between clients that publish messages and clients that subscribe, forwarding all received messages to matching subscriber clients. Sometimes we also directly refer to the server as a Broker.

### Main Functions of Server

- **Message Routing**: Forward messages to appropriate subscribers based on topics
- **Connection Management**: Manage client connections and disconnections
- **Session Management**: Maintain client session states
- **Security Control**: Provide authentication and authorization mechanisms
- **Message Storage**: Store retained messages and offline messages

## Client

A device or application that uses the MQTT protocol to connect to an MQTT server. It can be a publisher, a subscriber, or both.

### Types of Clients

- **Publisher**: Only publishes messages, doesn't subscribe to topics
- **Subscriber**: Only subscribes to topics, doesn't publish messages
- **Publisher-Subscriber**: Can both publish messages and subscribe to topics

### Client Lifecycle

1. **Connect**: Establish connection with MQTT server
2. **Authenticate**: Perform identity verification
3. **Subscribe**: Subscribe to topics of interest
4. **Publish/Receive**: Publish messages or receive subscribed messages
5. **Disconnect**: Disconnect from server

## Topic

Topics are used to identify and distinguish different messages. They are the foundation of MQTT message routing. Publishers can specify the message topic when publishing, and subscribers can choose to subscribe to topics they're interested in to receive relevant messages.

### Topic Naming Rules

- **UTF-8 Encoding**: Topic names use UTF-8 encoding
- **Hierarchical Structure**: Use slashes `/` to separate different levels
- **Length Limit**: Topic names cannot exceed 65535 bytes
- **Case Sensitive**: Topic names are case sensitive

### Topic Hierarchy Examples

```text
sensor/temperature/room1
sensor/humidity/room1
device/status/machine1
home/lighting/livingroom
```

## Wildcards

Subscribers can use wildcards in subscribed topics to achieve the purpose of subscribing to multiple topics at once. MQTT provides two types of topic wildcards: single-level wildcards and multi-level wildcards to meet different subscription needs.

### Single-Level Wildcard `+`

- Matches one topic level
- Must occupy the entire level
- Can be used in multiple levels

**Example**:

```text
sensor/+/temperature  # Matches sensor/room1/temperature, sensor/room2/temperature
```

### Multi-Level Wildcard `#`

- Matches any number of topic levels
- Must be the last character of the topic
- Can match zero or more levels

**Example**:

```text
sensor/#  # Matches sensor, sensor/temperature, sensor/room1/temperature
```

## QoS

MQTT defines three QoS levels to provide different message reliability guarantees. Each message can independently set its own QoS when publishing.

### QoS 0 - At Most Once

- **Characteristics**: Messages may be lost but won't be duplicated
- **Use Cases**: Scenarios where message loss is not critical
- **Performance**: Highest performance, lowest latency

### QoS 1 - At Least Once

- **Characteristics**: Messages are guaranteed to arrive but may be duplicated
- **Use Cases**: Scenarios that require guaranteed message delivery
- **Performance**: Medium performance, requires acknowledgment mechanism

### QoS 2 - Exactly Once

- **Characteristics**: Messages are guaranteed to arrive and won't be duplicated
- **Use Cases**: Scenarios with the highest message reliability requirements
- **Performance**: Lowest performance, requires four-way handshake

### QoS Selection Recommendations

- **QoS 0**: Sensor data, status updates
- **QoS 1**: Control commands, important notifications
- **QoS 2**: Critical configurations, financial transactions

## Session

QoS only designs the theoretical mechanism for reliable message delivery, while sessions ensure that the protocol flow of QoS 1 and 2 can be truly implemented.

### Session Types

#### Temporary Session

- **Characteristics**: Only lasts as long as the network connection
- **Use Cases**: Simple applications that don't need persistence
- **Advantages**: Low resource usage, simple connection

#### Persistent Session

- **Characteristics**: Exists across multiple network connections
- **Use Cases**: Applications that need offline message storage
- **Advantages**: Supports offline messages, connection recovery

### Session Recovery

- **Clean Session = true**: Create new session
- **Clean Session = false**: Restore existing session

## Retained Messages

Unlike ordinary messages, retained messages can be retained in the MQTT server. Any new subscriber subscribing to a topic that matches the topic in the retained message will immediately receive that message, even if the message was published before they subscribed to the topic.

### Characteristics of Retained Messages

- **Persistent Storage**: Messages are stored in the server
- **Immediate Access**: New subscribers immediately receive the latest message
- **Topic Uniqueness**: Each topic can only have one retained message
- **Automatic Override**: New retained messages override old ones

### Application Scenarios for Retained Messages

- **Device Status**: Store the latest status of devices
- **Configuration Information**: Store system configuration parameters
- **Real-time Data**: Store the latest readings from sensors
- **System Notifications**: Store important system notifications

## Will Messages

The characteristics of the publish-subscribe pattern determine that except for the server, no client can perceive when a client leaves the communication network. Will messages provide clients that disconnect unexpectedly with the ability to notify other clients.

### Characteristics of Will Messages

- **Automatic Trigger**: Automatically sent when client disconnects abnormally
- **Configurable**: Can set topic, content, QoS, etc.
- **Immediate**: Sent immediately or delayed after disconnection
- **Reliable**: Ensures important status information is delivered

### Application Scenarios for Will Messages

- **Device Offline Detection**: Monitor device connection status
- **Fault Alerts**: Send alerts when devices fail
- **Status Synchronization**: Maintain system status consistency
- **Resource Cleanup**: Release resources occupied by offline devices

## Shared Subscription

By default, messages are forwarded to all matching subscribers. But sometimes, we may want multiple clients to collaboratively process received messages to improve load capacity through horizontal scaling.

### Characteristics of Shared Subscription

- **Load Balancing**: Messages are load-balanced within subscription groups
- **Group Isolation**: Different subscription groups are independent
- **High Availability**: Supports dynamic client joining and leaving
- **Horizontal Scaling**: Scale processing capacity by adding subscribers

### Shared Subscription Syntax

- **With Groups**: `$share/{group}/{topic}`
- **Without Groups**: `$queue/{topic}`

### Application Scenarios for Shared Subscription

- **Task Distribution**: Distribute tasks to multiple worker nodes
- **Message Queuing**: Implement message queue-like functionality
- **Load Balancing**: Improve message processing capacity
- **High Availability**: Provide failover capabilities

## $SYS Topics

Topics prefixed with `$SYS/` are reserved for the server to publish specific messages, such as server uptime, client connection/disconnection event notifications, current number of connected clients, etc.

### Characteristics of System Topics

- **Server Published**: Automatically published by MQTT server
- **Real-time Information**: Provides server runtime status information
- **Monitoring Data**: Contains performance metrics and statistics
- **Event Notifications**: Client connection/disconnection events

### Common System Topics

```text
$SYS/broker/uptime                    # Server uptime
$SYS/broker/clients/connected         # Current number of connected clients
$SYS/broker/clients/disconnected      # Number of disconnected clients
$SYS/broker/messages/received         # Number of received messages
$SYS/broker/messages/sent             # Number of sent messages
```

### Applications of System Topics

- **System Monitoring**: Monitor server runtime status
- **Performance Analysis**: Analyze system performance metrics
- **Alert Notifications**: Receive system alert information
- **Operations Management**: Perform system operations management

## Summary

MQTT core concepts form the foundation of the MQTT protocol. Understanding these concepts is crucial for correctly using the MQTT protocol:

- **Publish-Subscribe Pattern** provides flexible message delivery mechanisms
- **Topics and Wildcards** implement flexible message routing
- **QoS Mechanisms** guarantee different levels of message reliability
- **Session Management** ensures reliable message transmission
- **Retained Messages** provide immediate data access capabilities
- **Will Messages** implement device status monitoring
- **Shared Subscription** provides load balancing and high availability
- **System Topics** provide server monitoring capabilities

These concepts work together to form a complete, flexible, and reliable MQTT message delivery system.
