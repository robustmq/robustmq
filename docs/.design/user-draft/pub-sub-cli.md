1. cli-command 实现订阅和发布基础命令


    `publish`

    ```console
    % cli-command mqtt --server=127.0.0.1:1883 publish --username=admin --password=pwd123 --topic=test/topic1 --qos=0
    able to connect: "127.0.0.1:1883"
    you can post a message on the terminal:
    1
    > You typed: 1
    2
    > You typed: 2
    3
    > You typed: 3
    4
    > You typed: 4
    5
    > You typed: 5
    ^C>  Ctrl+C detected,  Please press ENTER to end the program.
    ```

    `subscribe`

    ```console
    % cli-command mqtt --server=127.0.0.1:1883 subscribe --username=admin --password=pwd123 --topic=test/topic1 --qos=0

    able to connect: "127.0.0.1:1883"
    subscribe success
    payload: 1
    payload: 2
    payload: 3
    payload: 4
    payload: 5
    ^C Ctrl+C detected,  Please press ENTER to end the program.
    End of input stream.
    ```

2. 参数列表

    | 参数名称        | 参数含义           |  long           | short          | 默认值          |
    |----------------|------------------|-----------------|-----------------|----------------|
    | publish        | 发布消息          |                  |                |                |
    | subscribe      | 订阅消息          |                  |                |                |
    | server         | mqttbroker地址   | server           | s              | 127.0.0.1:1883 |
    | topic          | 消息主题          | topic            | t              | /tests/t1      |
    | qos            | 消息质量等级       | qos              | q              | 1              |
    | username       | 用户名            | username         | u              | 7355608        |
    | password       | 密码              | password         | p              | 7355608        |
