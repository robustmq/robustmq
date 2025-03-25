# 集群模式

1. 前面部分和单机模式一样，参考 [单机模式](./Run-Standalone-Mode.md)。集群模式同样要启动两个组件：Placement-Center 和 MQTT-Broker。

> 集群模式和单机模式的区别就是，集群模式需要启动多个 Placement Center 和 MQTT Broker 节点。多个节点启动后，会自动组建成一个高可用的集群。

2. 启动多台 Placement-Center

```shell
$ bin/robust-server place start config/example/mqtt-cluster/placement-center/node-1.toml
Starting placement-center with config: config/example/mqtt-cluster/placement-center/node-1.toml
Config:config/example/mqtt-cluster/placement-center/node-1.toml
placement-center started successfully.
```

依次启动：node2 和 node3

```shell
$ bin/robust-server place start config/example/mqtt-cluster/placement-center/node-2.toml
$ bin/robust-server place start config/example/mqtt-cluster/placement-center/node-3.toml
```

3. 查看 Placement Center 集群状态

```
$ bin/robust-ctl place status
{"running_state":{"Ok":null},"id":1,"current_term":35,"vote":{"leader_id":{"term":35,"node_id":1},"committed":true},"last_log_index":1,"last_applied":{"leader_id":{"term":35,"node_id":1},"index":1},"snapshot":null,"purged":null,"state":"Leader","current_leader":1,"millis_since_quorum_ack":2,"last_quorum_acked":1742008607078799375,"membership_config":{"log_id":{"leader_id":{"term":0,"node_id":0},"index":0},"membership":{"configs":[[1,2,3]],"nodes":{"1":{"node_id":1,"rpc_addr":"127.0.0.1:1228"},"2":{"node_id":2,"rpc_addr":"127.0.0.1:2228"},"3":{"node_id":3,"rpc_addr":"127.0.0.1:3228"}}}},"heartbeat":{"1":1742008607081029875,"2":1742008607078799083,"3":1742008607078799083},"replication":{"1":{"leader_id":{"term":35,"node_id":1},"index":1},"2":{"leader_id":{"term":35,"node_id":1},"index":1},"3":{"leader_id":{"term":35,"node_id":1},"index":1}}}
```

4. 启动多台 MQTT-Broker

```shell
$ bin/robust-server mqtt start config/example/mqtt-cluster/mqtt-server/node-1.toml
Starting mqtt-server with config: config/example/mqtt-cluster/mqtt-server/node-1.toml
Config:config/example/mqtt-cluster/mqtt-server/node-1.toml
mqtt-server started successfully.
```

依次启动 broker2 和 broker3

```shell
$ bin/robust-server mqtt start config/example/mqtt-cluster/mqtt-server/node-2.toml
$ bin/robust-server mqtt start config/example/mqtt-cluster/mqtt-server/node-3.toml
```

5. 查看 MQTT Broker 集群状态

```shell
$ bin/robust-ctl mqtt status
cluster name: mqtt-broker
node list:
- 172.20.10.5@1
- 172.20.10.5@2
- 172.20.10.5@3
MQTT broker cluster up and running
```

5. MQTT 验证、查看日志、停止服务，参考 [单机模式](./Run-Standalone-Mode.md)。
