# Configuration Description

## Basic Configuration
```
# Define the name of the MQTT Broker cluster, used for identification and management
cluster_name = "mqtt-broker"

# Set the unique identifier for the current Broker, used to distinguish different Broker nodes within the cluster
broker_id = 1

# The port number for the gRPC service, default 9981
grpc_port = 9981

# The port number for the HTTP service, default 9982, provides the ability to query the current running status
http_port = 9982

# Metadata service address, multiple can be configured
placement_center = ["localhost:1228"]

# Define the timeout for MQTT Broker to report heartbeat information, default 15s
# Supports "HumanReadable DurationFormat", for example: 1m15s, 45s, 1h30m
heartbeat_timeout = 10s
```

## Network Configuration
```
[network]
# MQTT protocol, default 1883 and 8883
tcp_port = 1883
tcps_port = 8883

# WebSocket protocol, default 8083 and 8084
websocket_port = 8083
websockets_port = 8084

# QUIC UDP protocol, default 9083
quic_port = 9083

# Set the certificate and key for TLS secure communication, default no certificate
tls_cert = "./config/example/certs/cert.pem"
tls_key = "./config/example/certs/key.pem"
```

## TCP Protocol Related Configuration
```
[tcp_thread]
# The number of threads accepting client connections, default 1
accept_thread_num = 1

# The number of threads processing client requests, default 1
handler_thread_num = 10

# The number of threads sending responses to clients, default 1
response_thread_num = 1

# Maximum connection limit, default 1000
max_connection_num = 1000

# Request queue size, default 2000
request_queue_size = 2000

# Response queue size, default 2000
response_queue_size = 2000

# Maximum number of attempts to acquire a lock, default 30
lock_max_try_mut_times = 30

# Time interval between attempts to acquire a lock (milliseconds), default 50
lock_try_mut_sleep_time_ms = 50
```

## System Configuration
```
[system]
# The number of runtime worker threads, default 16
runtime_worker_threads = 128
# Default username
default_user = "admin"
# Default robustmq
default_password = "pwd123"
```

## Storage Configuration
```
[storage]
# Storage type, default is memory, supports memory, mysql
storage_type = "memory"
mysql_addr = ""
```

## Authentication Configuration
```
[auth]
storage_type = "placement"
journal_addr = ""
mysql_addr = ""
```

## Log Configuration
```
[log]
# Log configuration file path, default ./config/log4rs.yaml
log_config = "./config/log4rs.yaml"
# Log file storage path, current ./logs directory
log_path = "./robust-data/mqtt-broker/logs"
```
