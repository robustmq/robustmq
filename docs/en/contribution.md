## Build the development environment
### Install Rust base

Reference: https://course.rs/first-try/installation.html

### Install Cmake.

The mac installation command is as follows:
```
brew install cmake
```

### Install RocksDB

See documentation at https://github.com/rust-rocksdb/rust-rocksdb Installing rocksdb.

The mac installation command is as follows:
```
brew install rocksdb
```

## Run RobustMQ

### Run standalone by placement-center
```
cargo run --package cmd --bin placement-center -- --conf=config/placement-center.toml
```
The following output indicates that placement-center started successfully
```
2024-08-09 09:09:33 INFO Raft Node inter-node communication management thread started successfully
2024-08-09 09:09:33 INFO Starts the thread that sends Raft messages to other nodes
2024-08-09 09:09:33 INFO RobustMQ Meta Grpc Server start success. bind addr:0.0.0.0:1228
2024-08-09 09:09:33 INFO Placement Center HTTP Server start success. bind addr:0.0.0.0:1227
2024-08-09 09:09:34 INFO Node Raft Role changes from  【Follower】 to 【Leader】
```

### Run standalone by mqtt-server
```
cargo run --package cmd --bin mqtt-server -- --conf=config/mqtt-server.toml
```
The following output indicates that the mqtt-server has been started successfully:
```
2024-08-09 09:09:49 INFO Node 1 has been successfully registered
2024-08-09 09:09:49 INFO Broker Grpc Server start success. port:9981
2024-08-09 09:09:49 INFO Subscribe manager thread started successfully.
2024-08-09 09:09:49 INFO MQTT TCP Server started successfully, listening port: 1883
2024-08-09 09:09:49 INFO Broker WebSocket Server start success. port:8083
2024-08-09 09:09:49 INFO Broker HTTP Server start success. bind addr:9982
2024-08-09 09:09:49 INFO Broker WebSocket TLS Server start success. port:8084
2024-08-09 09:09:49 INFO MQTT TCP TLS Server started successfully, listening port: 8883
```

## Creating an ISSUE
Issues are of two types, requirements and MINOR Fixed, so at the header level they are distinguished by the prefixes: RBIP-* and minor.

- RBIP-* : This indicates that a feature or functionality has been added, such as RBIP-09 or RBIP-10, followed by an increasing number.
[image1](../images/image-1.png)
- MINOR: Identifies minor fixes or additions. You can start with MINOR: followed by a title.
[image1](../images/image-2.png)

## Create a Pull Request
If the PR has an ISSUE associated with it, it must be appended to the content of the PR:
```
close #issue_number
```

close is a fixed prefix, # is a fixed prefix, and issue_number represents the ISSUE number associated with this PR. For example:
[image1](../images/image-5.png)
#297, #292 are the ISSUE numbers.

For example, if you need to submit a PR to resolve ISSUE #297, the PR content needs to include
```
close #297
```

In this case, when the PR is merged, the ISSUE is automatically closed.After PR is merged, the effect of PR and ISSUE is as follows:
- PR
[image1](../images/image-3.png)
- ISSUE
[image1](../images/image-4.png)