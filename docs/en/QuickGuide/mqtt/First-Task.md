# First task

## Requirements
1. Getting Package

   You can download the package from the Github home page or compile the source code.

- Github Homepage Download： https://github.com/robustmq/robustmq/releases
- Compiling the source code： [Build from Source](./Build.md)

2. Unzip the installation package

```shell
$ tar -xzvf robustmq-v0.1.14-release.tar.gz
$ cd robustmq-v0.1.14-release
```


## 1. Start `Placement-Center` 
```shell
$ bin/robust-server place start

Starting placement-center with config: /Users/bytedance/test/robustmq-0.1.14/bin/../config/placement-center.toml
Config:/Users/bytedance/test/robustmq-0.1.14/bin/../config/placement-center.toml
placement-center started successfully.
```

## 2. Start `MQTT-Broker` 
```shell
$ bin/robust-server mqtt start

Starting mqtt-server with config: /Users/bytedance/test/robustmq-0.1.14/bin/../config/mqtt-server.toml
Config:/Users/bytedance/test/robustmq-0.1.14/bin/../config/mqtt-server.toml
mqtt-server started successfully.
```

## 3.Querying service status
```shell
# Check the Placement-Center status
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

# Check the MQTT-Broker status
$ ./bin/robust-ctl mqtt status

cluster name: mqtt-broker
node list:
- 10.225.110.179@1
MQTT broker cluster up and running
```

## 4. Try to post new information

### 4.1 Create User
MQTT Broker enables user authentication, where a client must provide a valid username and password to be authenticated before Posting or subscribable messages. Clients that fail to authenticate will not be able to communicate with the Broker.

Create a foo user using 'robust-ctl'
```shell
$ ./bin/robust-ctl mqtt user create --username foo --password 123
Created successfully!
```

### 4.2  Publish Messages
Use 'robust-ctl' to simulate the producer to post new messages on topic=demo


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

### 4.3 Subscribe Messages
Use 'robust-ctl' to simulate a consumer subscribe to topic=demo


```shell
$ ./bin/robust-ctl mqtt --server=localhost:1883 subscribe --topic=demo --qos=0 --username foo --password 123

able to connect: "localhost:1883"
subscribe success
payload: hello
payload: robustmq!
^C Ctrl+C detected,  Please press ENTER to end the program.
```

## Next Step

Learn more about RobustMQ concepts, configuration, and usage from the following page:
* [RobustMQ System Architecture](../../Architect/Overview.md)
* [RobustMQ Mqtt Config](../../Architect/Configuration/Mqtt-Server.md)
* [RobustMQ Mqtt Component](../../RobustMQ-MQTT/Overview.md)
