1. docker 构建镜像

    构建镜像需要在`robustmq`根目录构建

    构建 placement center 镜像
    ```shell
        docker build --target placement-center -t placement-center-test:0.1 .
    ```

    构建 mqserver 镜像
    ```shell
        docker build --target mqserver -t mqserver-test:0.1 .
    ```

2. docker 运行

    ```shell
    cd ./example/test-network-docker
    docker-compose -f compose/compose-test-net.yaml up

    ```

3. 简单使用

    更多命令请参考 [cli-command](../RobustMQ-Command/Mqtt-Broker.md)

    查看用户
    ```console
    % cli-command mqtt user list
    +----------+--------------+
    | username | is_superuser |
    +----------+--------------+
    | admin    | true         |
    +----------+--------------+
    ```

    发布消息
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

    订阅消息
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
4. 出现的问题怎么解决

   4.1 出现`Not found leader`这种问题是因为 `placement-center` 集群的数据有问题，需要清理掉 volumes

   ```console
    mqtt-server-node-1       | thread 'main' panicked at src/mqtt-broker/src/handler/cache.rs:523:17:
    mqtt-server-node-1       | Not found leader
    mqtt-server-node-1       | note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    mqtt-server-node-1 exited with code 101
   ```
    解决办法
    ```console
    % docker volume ls
    DRIVER    VOLUME NAME
    local     compose_mqtt-server-node-1-data
    local     compose_mqtt-server-node-2-data
    local     compose_mqtt-server-node-3-data
    local     compose_placement-center-node-1-data
    local     compose_placement-center-node-2-data
    local     compose_placement-center-node-3-data

    % docker volume rm compose_mqtt-server-node-1-data compose_placement-center-node-1-data compose_placement-center-node-2-data compose_placement-center-node-3-data
    ```
