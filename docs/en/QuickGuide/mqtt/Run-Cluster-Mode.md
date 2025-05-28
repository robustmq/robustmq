# Cluster mode

1. The front part is the same as the stand-alone mode，Reference[Standalone Mode](./Run-Standalone-Mode.md)。Cluster mode also starts two components ：Placement-Center and MQTT-Broker.

> The difference between cluster mode and standalone mode is that cluster mode needs to launch multiple Placement Center and MQTT Broker nodes. When multiple nodes are started, they are automatically formed into a highly available cluster.

2. Start multiple placement-centers

```shell
$ bin/robust-server place start config/example/mqtt-cluster/placement-center/node-1.toml
Starting placement-center with config: config/example/mqtt-cluster/placement-center/node-1.toml
Config:config/example/mqtt-cluster/placement-center/node-1.toml
placement-center started successfully.
```

Start sequentially: node2 and node3

```shell
$ bin/robust-server place start config/example/mqtt-cluster/placement-center/node-2.toml
$ bin/robust-server place start config/example/mqtt-cluster/placement-center/node-3.toml
```

3. Check the Placement Center cluster status

```shell
$ bin/robust-ctl place status
{"running_state":{"Ok":null},"id":1,"current_term":35,"vote":{"leader_id":{"term":35,"node_id":1},"committed":true},"last_log_index":1,"last_applied":{"leader_id":{"term":35,"node_id":1},"index":1},"snapshot":null,"purged":null,"state":"Leader","current_leader":1,"millis_since_quorum_ack":2,"last_quorum_acked":1742008607078799375,"membership_config":{"log_id":{"leader_id":{"term":0,"node_id":0},"index":0},"membership":{"configs":[[1,2,3]],"nodes":{"1":{"node_id":1,"rpc_addr":"127.0.0.1:1228"},"2":{"node_id":2,"rpc_addr":"127.0.0.1:2228"},"3":{"node_id":3,"rpc_addr":"127.0.0.1:3228"}}}},"heartbeat":{"1":1742008607081029875,"2":1742008607078799083,"3":1742008607078799083},"replication":{"1":{"leader_id":{"term":35,"node_id":1},"index":1},"2":{"leader_id":{"term":35,"node_id":1},"index":1},"3":{"leader_id":{"term":35,"node_id":1},"index":1}}}
```

4. Start multiple MQTT-Brokers

```shell
$ bin/robust-server mqtt start config/example/mqtt-cluster/mqtt-server/node-1.toml
Starting mqtt-server with config: config/example/mqtt-cluster/mqtt-server/node-1.toml
Config:config/example/mqtt-cluster/mqtt-server/node-1.toml
mqtt-server started successfully.
```

Start broker2 and broker3 in turn

```shell
$ bin/robust-server mqtt start config/example/mqtt-cluster/mqtt-server/node-2.toml
$ bin/robust-server mqtt start config/example/mqtt-cluster/mqtt-server/node-3.toml
```

5. View MQTT Broker cluster status

```shell
$ bin/robust-ctl mqtt status
cluster name: mqtt-broker
node list:
- 172.20.10.5@1
- 172.20.10.5@2
- 172.20.10.5@3
MQTT broker cluster up and running
```

5.MQTT validation, view log, stop service ，Reference[Standalone Mode](./Run-Standalone-Mode.md).
