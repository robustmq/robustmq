# 集群模式
1. Download .tar.gz
```
$ tar -xzvf robustmq-v0.0.1-release.tar.gz
$ cd robustmq-v0.0.1-release
```

2. Start Placement Center
```
$ bin/robust-server placement-center start config/cluster/placement-center/node-1.toml
$ bin/robust-server placement-center start config/cluster/placement-center/node-2.toml
$ bin/robust-server placement-center start config/cluster/placement-center/node-3.toml
```

3. Start MQTT Broker
```
$ bin/robust-server broker-mqtt start config/cluster/mqtt-server/node-1.toml
$ bin/robust-server broker-mqtt start config/cluster/mqtt-server/node-2.toml
$ bin/robust-server broker-mqtt start config/cluster/mqtt-server/node-3.toml
```
