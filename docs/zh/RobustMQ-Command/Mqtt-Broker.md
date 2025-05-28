# MQTT Broker Command

## 1. 集群状态

MQTT Broker 提供了集群状态查询功能，可以通过命令行工具查看集群的健康状态、节点信息等。

```console
% ./bin/robust-ctl mqtt status
cluster name: example_cluster
node list:
- node1
- node2
- node3
MQTT broker cluster up and running
```

## 2. 用户管理

MQTT Broker 启用了用户验证功能，客户端在发布或订阅消息前，
必须提供有效的用户名和密码以通过验证。
未通过验证的客户端将无法与 Broker 通信。
这一功能可以增强系统的安全性，防止未经授权的访问。

### 2.1 创建用户

创建新的 MQTT Broker 用户。

```console
% ./bin/robust-ctl mqtt user create --username=testp --password=7355608 --is_superuser=false
Created successfully!
```

### 2.2 删除用户

删除已有的 MQTT Broker 用户。

```console
% ./bin/robust-ctl mqtt user delete --username=testp
Deleted successfully!
```

### 2.3 用户列表

列出所有已创建的用户。

```console
% ./bin/robust-ctl mqtt user list
+----------+--------------+
| username | is_superuser |
+----------+--------------+
| admin    | true         |
+----------+--------------+
| testp    | false        |
+----------+--------------+
```

## 3. 发布、订阅消息

### 3.1 发布 MQTT 消息

```console
% ./bin/robust-ctl mqtt --server=localhost:1883 publish --username=admin --password=pwd123 --topic=test/topic1 --qos=0
able to connect: "localhost:1883"
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

### 3.2 订阅 MQTT 消息

```console
% ./bin/robust-ctl mqtt --server=localhost:1883 subscribe --username=admin --password=pwd123 --topic=test/topic1 --qos=0

able to connect: "localhost:1883"
subscribe success
payload: 1
payload: 2
payload: 3
payload: 4
payload: 5
^C Ctrl+C detected,  Please press ENTER to end the program.
End of input stream.
```

### 3.3 发布保留消息

```console
% ./bin/robust-ctl mqtt --server=localhost:1883 publish --username=admin --password=pwd123 --topic=\$share/group1/test/topic1 --qos=1 --retained
able to connect: "localhost:1883"
you can post a message on the terminal:
helloworld!
> You typed: helloworld!
published retained message
```

```console
% ./bin/robust-ctl mqtt --server=localhost:1883 subscribe --username=admin --password=pwd123 --topic=\$share/group1/test/topic1 --qos=0
able to connect: "localhost:1883"
subscribe success
Retain message: helloworld!
```

## 4. ACL（访问控制列表）管理

### 4.1 创建 ACL

创建新的 ACL 规则。

```console
% ./bin/robust-ctl mqtt --server=localhost:1883 acl create --cluster-name=admin --acl=xxx
able to connect: "localhost:1883"
Created successfully!
```

### 4.2 删除 ACL

删除已有的 ACL 规则。

```console
% ./bin/robust-ctl mqtt --server=localhost:1883 acl delete --cluster-name=admin --acl=xxx
able to connect: "localhost:1883"
Deleted successfully!
```

### 4.3 ACL 列表

列出所有已创建的 ACL 规则。

```console
% ./bin/robust-ctl mqtt --server=localhost:1883 acl list
+---------------+---------------+-------+----+--------+------------+
| resource_type | resource_name | topic | ip | action | permission |
+---------------+---------------+-------+----+--------+------------+
```

## 5. 黑名单管理

### 5.1 创建黑名单

创建新的黑名单规则。

```console
% ./bin/robust-ctl mqtt --server=localhost:1883 blacklist create --cluster-name=admin --blacklist=client_id
able to connect: "localhost:1883"
Created successfully!
```

### 5.2 删除黑名单

删除已有的黑名单规则。

```console
% ./bin/robust-ctl mqtt --server=localhost:1883 blacklist delete --cluster-name=admin --blacklist-type=client_id --resource-name=client1
able to connect: "localhost:1883"
Deleted successfully!
```

### 5.3 黑名单列表

列出所有已创建的黑名单规则。

```console
% ./bin/robust-ctl mqtt --server=localhost:1883 blacklist list
+----------------+---------------+----------+------+
| blacklist_type | resource_name | end_time | desc |
+----------------+---------------+----------+------+
```

## 6. 开启慢订阅功能

### 6.1 开启/关闭慢订阅

慢订阅统计功能主要是为了在消息到达 Broker 后，
Broker 来计算完成消息处理以及传输整个流程所消耗的时间(时延),
如果时延超过阈值，我们就会记录一条相关的信息在集群慢订阅日志当中，
运维人员可以通过命令查询整个集群下的慢订阅记录信息，
通过慢订阅信息来解决。

- 开启慢订阅

```console
% ./bin/robust-ctl mqtt slow-sub --enable=true
The slow subscription feature has been successfully enabled.
```

- 关闭慢订阅

```console
% ./bin/robust-ctl mqtt slow-sub --enable=false
The slow subscription feature has been successfully closed.
```

### 6.2 查询慢订阅记录

当我们启动了慢订阅统计功能之后, 集群就开启慢订阅统计功能，
这样我们可以通过对应的命令来去查询对应的慢订阅记录，
如果我们想要查看慢订阅记录，客户端可以输入如下命令

```console
% ./bin/robust-ctl mqtt slow-sub --query=true
+-----------+-------+----------+---------+-------------+
| client_id | topic | sub_name | time_ms | create_time |
+-----------+-------+----------+---------+-------------+
```

如果想要获取更多的慢订阅记录，
并且想要按照从小到大的顺序进行升序排序，
那么可以使用如下的命令

```console
% ./bin/robust-ctl mqtt slow-sub --list=200 --sort=asc
+-----------+-------+----------+---------+-------------+
| client_id | topic | sub_name | time_ms | create_time |
+-----------+-------+----------+---------+-------------+
```

对于慢订阅查询，我们同样支持筛选查询功能，我们支持使用 topic,
sub_name 以及 client_id 的方式来获取不同字段过滤后的结果，
其结果默认从大到小倒序排序，参考使用命令如下

```console
% ./bin/robust-ctl mqtt slow-sub --topic=topic_test1 --list=200
+-----------+-------+----------+---------+-------------+
| client_id | topic | sub_name | time_ms | create_time |
+-----------+-------+----------+---------+-------------+
```

## 7. 主题重写规则

很多物联网设备不支持重新配置或升级，修改设备业务主题会非常困难。

主题重写功能可以帮助使这种业务升级变得更容易：通过设置一套规则，它可以在订阅、发布时改变将原有主题重写为新的目标主题。

### 7.1 创建主题重写规则

```console
% ./bin/robust-ctl mqtt topic-rewrite create --action=xxx --source-topic=xxx --dest-topic=xxx --regex=xxx
Created successfully!
```

### 7.2 删除主题重写规则

```console
% ./bin/robust-ctl mqtt topic-rewrite delete --action=xxx --source-topic=xxx
Deleted successfully!
```

## 8. 连接抖动检测

在黑名单功能的基础上，支持自动封禁那些被检测到短时间内频繁登录的客户端，并且在一段时间内拒绝这些客户端的登录，以避免此类客户端过多占用服务器资源而影响其他客户端的正常使用。

- 开启连接抖动检测

```console
% ./bin/robust-ctl mqtt flaping-detect --is-enable=false --window-time=1 --max-client-connections=15 --ban-time=5
The flapping detect feature has been successfully enabled.
```

- 关闭连接抖动检测

```console
% ./bin/robust-ctl mqtt flaping-detect --is-enable=false
The flapping detect feature has been successfully closed.
```

## 9. 连接

连接列表命令用于查询 MQTT Broker 当前的连接状态，提供连接 ID、类型、协议、源地址等相关信息。

```console
% ./bin/robust-ctl mqtt list-connection
connection list:
+---------------+-----------------+----------+-------------+------+
| connection_id | connection_type | protocol | source_addr | info |
+---------------+-----------------+----------+-------------+------+
```

## 10. 主题

查看当前系统中所有订阅的主题。 list-topic 列出所有主题,该命令可用于监视主题的数量和分布。

```console
% ./bin/robust-ctl mqtt list-topic
topic list result:
+----------------------------------+---------------------------------------------------------+--------------+---------------------------+
| topic_id                         | topic_name                                              | cluster_name | is_contain_retain_message |
+----------------------------------+---------------------------------------------------------+--------------+---------------------------+
| b63fc4d3523644e1b1da0149bb376c74 | $SYS/brokers/10.7.141.123/version                       | mqtt-broker  | false                     |
+----------------------------------+---------------------------------------------------------+--------------+---------------------------+
......
```
