## 1. 构建 Docker 镜像
构建镜像需要在`robustmq`根目录构建

- 构建 placement center 镜像

```shell
docker build --target placement-center -t placement-center-test:0.1 .
```

- 构建 mqserver 镜像

```shell
docker build --target mqserver -t mqserver-test:0.1 .
```

## 2. docker 运行

```shell
cd ./example/test-network-docker
docker-compose -f compose/compose-test-net.yaml up
```

## 3. 验证 MQTT 功能是否正常
   
查看文档执行测试：[MQTT 功能测试](./MQTT-test.md)

    
## 4. 出现的问题怎么解决

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
