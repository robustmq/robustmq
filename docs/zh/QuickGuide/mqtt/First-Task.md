# 第一个任务

## 要求
1. 获取安装包
   可以通过在 Github 主页上下载安装包，也可以通过编译源码获取。

- Github 主页下载： https://github.com/robustmq/robustmq/releases
- 编译源码： [Build from Source](./Build.md)

2. 解压安装包

```
$ tar -xzvf robustmq-v0.1.14-release.tar.gz
$ cd robustmq-v0.1.14-release
```

## 1.启动 `Placement-Center` 组件
```shell
$ bin/robust-server place start

Starting placement-center with config: /Users/bytedance/test/robustmq-0.1.14/bin/../config/placement-center.toml
Config:/Users/bytedance/test/robustmq-0.1.14/bin/../config/placement-center.toml
placement-center started successfully.
```

## 2.启动 `MQTT-Broker` 组件
```shell
$ bin/robust-server mqtt start

Starting mqtt-server with config: /Users/bytedance/test/robustmq-0.1.14/bin/../config/mqtt-server.toml
Config:/Users/bytedance/test/robustmq-0.1.14/bin/../config/mqtt-server.toml
mqtt-server started successfully.
```

## 3.查询服务状态，确保服务正常运行
```shell
# 查看 Placement-Center 状态
$ ./bin/robust-ctl place status | jq

{
  "running_state": {
    "Ok": null
  },
  "id": 1,
  "current_term": 1,
  "vote": {
    "leader_id": {
      "term": 1,
      "node_id": 1
    },
    "committed": true
  },
  "last_log_index": 27,
  "last_applied": {
    "leader_id": {
      "term": 1,
      "node_id": 1
    },
    "index": 27
  },
  "snapshot": null,
  "purged": null,
  "state": "Leader",
  "current_leader": 1,
  "millis_since_quorum_ack": 0,
  "last_quorum_acked": 1745308291237907708,
  "membership_config": {
    "log_id": {
      "leader_id": {
        "term": 0,
        "node_id": 0
      },
      "index": 0
    },
    "membership": {
      "configs": [
        [
          1
        ]
      ],
      "nodes": {
        "1": {
          "node_id": 1,
          "rpc_addr": "localhost:1228"
        }
      }
    }
  },
  "heartbeat": {
    "1": 1745308290860261416
  },
  "replication": {
    "1": {
      "leader_id": {
        "term": 1,
        "node_id": 1
      },
      "index": 27
    }
  }
}

# 查看 MQTT-Broker 状态
$ ./bin/robust-ctl mqtt status

cluster name: mqtt-broker
node list:
- 10.225.110.179@1
MQTT broker cluster up and running
```

## 4.尝试发布新的信息

### 4.1 创建一个新用户用于发布/订阅信息

MQTT Broker 启用了用户验证功能，客户端在发布或订阅消息前， 必须提供有效的用户名和密码以通过验证。 未通过验证的客户端将无法与 Broker 通信。

使用 `robust-ctl` 创建一个 foo 用户
```shell
$ ./bin/robust-ctl mqtt user create --username foo --password 123
Created successfully!
```

### 4.2 生产者发布信息

使用 `robust-ctl` 模拟生产者在 topic=demo 上发布新消息

```shell
$ ./bin/robust-ctl mqtt --server=localhost:1883 publish --topic=demo --qos=0 --username foo --password 123

able to connect: "localhost:1883"
you can post a message on the terminal:
hello
> You typed: hello
robustmq!
> You typed: robustmq!
^C>  Ctrl+C detected,  Please press ENTER to end the program.
```

### 4.3 消费者订阅信息

使用 `robust-ctl` 模拟消费者在 topic=demo 上订阅信息

```shell
$ ./bin/robust-ctl mqtt --server=localhost:1883 subscribe --topic=demo --qos=0 --username foo --password 123

able to connect: "localhost:1883"
subscribe success
payload: hello
payload: robustmq!
^C Ctrl+C detected,  Please press ENTER to end the program.
```

## 下一步

从以下页面了解更多 RobustMQ 概念、配置和使用方式:
* [RobustMQ 系统架构概念](../../Architect/Overview.md)
* [RobustMQ Mqtt 配置](../../Architect/Configuration/Mqtt-Server.md)
* [RobustMQ Mqtt 组件](../../RobustMQ-MQTT/Overview.md)
