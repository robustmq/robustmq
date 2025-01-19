1. Docker Image Build

    Building images requires being done from the root directory of `robustmq`.

    Build the Placement Center image
    ```shell
        docker build --target placement-center -t placement-center-test:0.1 .
    ```

    Build the MQServer image
    ```shell
        docker build --target mqserver -t mqserver-test:0.1 .
    ```

2. Running with Docker

    ```shell
    cd ./example/test-network-docker
    docker-compose -f compose/compose-test-net.yaml up
    ```

3. Simple Usage

    For more commands, please refer to [cli-command](../RobustMQ-Command/Mqtt-Broker.md)

    List users
    ```console
    % cli-command mqtt user list
    +----------+--------------+
    | username | is_superuser |
    +----------+--------------+
    | admin    | true         |
    +----------+--------------+
    ```

    Publish messages
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

    Subscribe to messages
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

4. Troubleshooting

   4.1 The issue `Not found leader` occurs due to problems with the data in the `placement-center` cluster, which requires removing the volumes.

   ```console
    mqtt-server-node-1       | thread 'main' panicked at src/mqtt-broker/src/handler/cache.rs:523:17:
    mqtt-server-node-1       | Not found leader
    mqtt-server-node-1       | note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    mqtt-server-node-1 exited with code 101
   ```

    Solution

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
