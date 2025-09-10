## Endpoint

| feature | Describe |
| --- | --- |
| TCP  | 117.72.92.117:1883 |
| TCPS | 117.72.92.117:1884 |
| WebSocket | 117.72.92.117:1093 |
| WebSockets | 117.72.92.117:1094 |
| QUIC | 117.72.92.117:1083 |

## User/Password

robustmq/robustmq

## RobustMQ Dashboard

<http://117.72.92.117:3000/>

![image](../../images/dashboard.png)

## Try: Command Pub/Sub

- Pub

```
$ bin/robust-ctl mqtt --server=117.72.92.117:1883 publish --username=robustmq --password=robustmq --topic=/test/topic1 --qos=0
able to connect: "117.72.92.117:1883"
you can post a message on the terminal:
1
> You typed: 1
2
> You typed: 2
2
> You typed: 2
2
> You typed: 2
2
> You typed: 2
```

- Sub

```
$ bin/robust-ctl mqtt --server=117.72.92.117:1883 subscribe --username=robustmq --password=robustmq --topic=/test/topic1 --qos=0
able to connect: "117.72.92.117:1883"
subscribe success
payload: 1
payload: 2
payload: 2
payload: 2
payload: 2
payload: 2
payload: 2
payload: 2
```

## Try：MQTTX Pub/Sub

- Connect broker
![image](../../images/mqttx01.png)

- Pub/Sub
![image](../../images/mqttx-2.png)