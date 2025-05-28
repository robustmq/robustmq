1. Getting Package

   You can download the package from the Github home page or compile the source code.

- Github Homepage Download： https://github.com/robustmq/robustmq/releases
- Compiling the source code： [Build from Source](./Build.md)

2. Unzip the installation package

```shell
$ tar -xzvf robustmq-v0.1.14-release.tar.gz
$ cd robustmq-v0.1.14-release
```

3. Start Placement-Center

```shell
$ bin/robust-server place start
Starting placement-center with config: /Users/bytedance/test/robustmq-0.1.14/bin/../config/placement-center.toml
Config:/Users/bytedance/test/robustmq-0.1.14/bin/../config/placement-center.toml
placement-center started successfully.
```

4. Start MQTT-Broker

```shell
$ bin/robust-server mqtt start
Starting mqtt-server with config: /Users/bytedance/test/robustmq-0.1.14/bin/../config/mqtt-server.toml
Config:/Users/bytedance/test/robustmq-0.1.14/bin/../config/mqtt-server.toml
mqtt-server started successfully.
```

5. Checking service status

- View the Placement Cener running status

```shell
$ bin/robust-ctl place status
{"running_state":{"Ok":null},"id":1,"current_term":1,"vote":{"leader_id":{"term":1,"node_id":1},"committed":true},"last_log_index":28,"last_applied":{"leader_id":{"term":1,"node_id":1},"index":28},"snapshot":null,"purged":null,"state":"Leader","current_leader":1,"millis_since_quorum_ack":0,"last_quorum_acked":1742005289409447084,"membership_config":{"log_id":{"leader_id":{"term":0,"node_id":0},"index":0},"membership":{"configs":[[1]],"nodes":{"1":{"node_id":1,"rpc_addr":"127.0.0.1:1228"}}}},"heartbeat":{"1":1742005289032346459},"replication":{"1":{"leader_id":{"term":1,"node_id":1},"index":28}}}
```

- View the Placement Cener running status

```shell
$ bin/robust-ctl mqtt status
cluster name: mqtt-broker
node list:
- 172.20.10.5@1
MQTT broker cluster up and running
```

When the above information is displayed, the service is running correctly. At this point, the MQTT client can be used to connect to the service, Pub/Sub messages

6. Verify that MQTT functions correctly

Check the documentation to run the test：[MQTT functional tests](./MQTT-test.md)

7. Viewing logs

```shell
tail -fn 300 logs/placement-center-nohup.log
tail -fn 300 logs/mqtt-server-nohup.log
```

8. Stop Service

```shell
$ bin/robust-server place stop
$ bin/robust-server mqtt stop
```
