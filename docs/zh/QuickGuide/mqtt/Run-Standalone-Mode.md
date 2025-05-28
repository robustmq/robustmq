1. 获取安装包
   可以通过在 Github 主页上下载安装包，也可以通过编译源码获取。

- Github 主页下载： https://github.com/robustmq/robustmq/releases
- 编译源码： [Build from Source](./Build.md)

2. 解压安装包

```
$ tar -xzvf robustmq-v0.1.14-release.tar.gz
$ cd robustmq-v0.1.14-release
```

3. 启动 Placement-Center

```shell
$ bin/robust-server place start
Starting placement-center with config: /Users/bytedance/test/robustmq-0.1.14/bin/../config/placement-center.toml
Config:/Users/bytedance/test/robustmq-0.1.14/bin/../config/placement-center.toml
placement-center started successfully.
```

4. 启动 MQTT-Broker

```shell
$ bin/robust-server mqtt start
Starting mqtt-server with config: /Users/bytedance/test/robustmq-0.1.14/bin/../config/mqtt-server.toml
Config:/Users/bytedance/test/robustmq-0.1.14/bin/../config/mqtt-server.toml
mqtt-server started successfully.
```

5. 查看服务状态

- 查看 Placement Center 运行状态

```shell
$ bin/robust-ctl place status
{"running_state":{"Ok":null},"id":1,"current_term":1,"vote":{"leader_id":{"term":1,"node_id":1},"committed":true},"last_log_index":28,"last_applied":{"leader_id":{"term":1,"node_id":1},"index":28},"snapshot":null,"purged":null,"state":"Leader","current_leader":1,"millis_since_quorum_ack":0,"last_quorum_acked":1742005289409447084,"membership_config":{"log_id":{"leader_id":{"term":0,"node_id":0},"index":0},"membership":{"configs":[[1]],"nodes":{"1":{"node_id":1,"rpc_addr":"127.0.0.1:1228"}}}},"heartbeat":{"1":1742005289032346459},"replication":{"1":{"leader_id":{"term":1,"node_id":1},"index":28}}}
```

- 查看 MQTT-Broker 运行状态

```shell
$ bin/robust-ctl mqtt status
cluster name: mqtt-broker
node list:
- 172.20.10.5@1
MQTT broker cluster up and running
```

当显示上面信息时，就表示服务运行正常。此时，即可使用 MQTT 客户端连接服务，Pub/Sub 消息

6. 验证 MQTT 功能是否正常

查看文档执行测试：[MQTT 功能测试](./MQTT-test.md)

7. 查看日志

```shell
$ tail -fn 300 logs/placement-center-nohup.log
$ tail -fn 300 logs/mqtt-server-nohup.log
```

8. 停止服务

```shell
$ bin/robust-server place stop
$ bin/robust-server mqtt stop
```
