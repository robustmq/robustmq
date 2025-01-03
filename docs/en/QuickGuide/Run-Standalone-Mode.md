1. download robustmq-v0.0.1-release.tar.gz binary package

```shell
 tar -xzvf robustmq-v0.0.1-release.tar.gz
 cd robustmq-v0.0.1-release
 make build
 cd build
 tar -zxvf robustmq-local.tar.gz
 cd ../
 cp -r  build/robustmq-local/libs .
 cp -r example/mqtt-cluster/mqtt-server config/cluster
 cp -r example/mqtt-cluster/placement-center config/cluster
```

1. start Placement-Center

```shell
bin/robust-server place start config/placement-center.toml
```

3. start MQTT-Broker

```shell
bin/robust-server mqtt start config/mqtt-server.toml
```

1. stop services

```shell
bin/robust-server place stop
bin/robust-server mqtt stop
```

1. view logs

```shell
tail -fn 300 logs/placement-center-nohub.log
tail -fn 300 logs/mqtt-server-nohub.log
```
