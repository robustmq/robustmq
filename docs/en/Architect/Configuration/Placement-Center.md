# Configuration Description
## Cluster Name
```
# Define the cluster name, used to identify the current cluster, default name placement-center
cluster_name = "placement-test"
```

## Node-related Configuration
```
[node]
# Define the node ID, used to uniquely identify the current node, default is 1
node_id = 1

# Define the network address of the node, default is 127.0.0.1
addr = "127.0.0.1"

# Define the node list, including node ID and corresponding gRPC address
nodes = { 1 = "127.0.0.1:1228" }
```

## Network-related Configuration
```
[network]
# Define the gRPC service port of the node
grpc_port = 1228

# Define the HTTP service port of the node, used to obtain cluster status, node information, etc.
http_port = 1227
```

## Process-related Parameters
```
[system]
# Define the number of runtime threads, runtime_work_threads * 2
runtime_work_threads = 100
```

## Broker Node Heartbeat Detection Parameters
```
[heartbeat]
# Define the heartbeat timeout period in milliseconds, used to detect node failures, default is 30000
heartbeat_timeout_ms = 30000

# Define the heartbeat check interval in milliseconds, used for regular node status checks, default is 1000
heartbeat_check_time_ms = 1000
```

## RocksDB-related Configuration
```
[rocksdb]
# Define the data storage path for RocksDB
data_path = "./robust-data/placement-center/data"

# Configure the maximum number of open files for the RocksDB database, supports a large number of concurrent read operations, default is 10000
max_open_files = 10000
```

## Log Configuration, specifying log path and configuration file
```
[log]
# Log configuration file path, default is ./config/log4rs.yaml
log_config = "./config/log4rs.yaml"

# Log file storage directory, default directory is ./logs/placement-center
log_path = "./robust-data/placement-center/logs"
```
