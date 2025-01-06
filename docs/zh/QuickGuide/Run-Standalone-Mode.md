1. 下载 .tar.gz
```
$ tar -xzvf robustmq-v0.0.1-release.tar.gz
$ cd robustmq-v0.0.1-release
```
2. 编译

```shell

 make build
 cd build
 tar -zxvf robustmq-local.tar.gz
 cd ../
 cp -r  build/robustmq-local/libs .
 cp -r example/mqtt-cluster/mqtt-server config/cluster
 cp -r example/mqtt-cluster/placement-center config/cluster
```

3. 启动 Placement-Center

```shell
bin/robust-server place start config/placement-center.toml
```

4. 启动 MQTT-Broker

```shell
bin/robust-server mqtt start config/mqtt-server.toml
```

4. 停止服务

```shell
bin/robust-server place stop
bin/robust-server mqtt stop
```

5. 查看日志

```shell
tail -fn 300 logs/placement-center-nohub.log
tail -fn 300 logs/mqtt-server-nohub.log
```
