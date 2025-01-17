1. cli-command 实现订阅和发布基础命令

```shell
cli-command mqtt -s 127.0.0.1:9981 publish -u userp -p 7355608  -t /tests/t1 -m "mqtt message" -q 1
cli-command mqtt -s 127.0.0.1:9981 subscribe -u users -p 7355608 -t /tests/t1 -q 1
```

2. 参数列表

    | 参数名称        | 参数含义           |  long           | short          | 默认值          |
    |----------------|------------------|-----------------|-----------------|----------------|
    | publish        | 发布消息          |                  |                |                |
    | subscribe      | 订阅消息          |                  |                |                |
    | server         | mqttbroker地址   | server           | s              | 127.0.0.1:9981 |
    | topic          | 消息主题          | topic            | t              | /tests/t1      |
    | message        | 消息内容          | message          | m              | mqtt message   |
    | qos            | 消息质量等级       | qos              | q              | 1              |
    | username       | 用户名            | username         | u              | 7355608        |
    | password       | 密码              | password         | p              | 7355608        |
