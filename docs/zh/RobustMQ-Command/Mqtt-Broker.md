# MQTT Broker Command

## 1. 用户管理

### 1.1 创建用户

MQTT Broker 启用了用户验证功能，客户端在发布或订阅消息前，
必须提供有效的用户名和密码以通过验证。
未通过验证的客户端将无法与 Broker 通信。
这一功能可以增强系统的安全性，防止未经授权的访问。

```console
% ./bin/robust-ctl mqtt  mqtt user create --username=testp --password=7355608
Created successfully!
```

### 1.2 删除用户

```console
% ./bin/robust-ctl mqtt  mqtt user delete --username=testp
Deleted successfully!
```

### 1.3 用户列表

```console
% ./bin/robust-ctl mqtt  mqtt user list
+----------+--------------+
| username | is_superuser |
+----------+--------------+
| admin    | true         |
+----------+--------------+
| testp    | false        |
+----------+--------------+
```

## 2. MQTT 发布、订阅消息

### 2.1 发布 MQTT 消息

```console
    % ./bin/robust-ctl mqtt mqtt --server=127.0.0.1:1883 publish --username=admin --password=pwd123 --topic=test/topic1 --qos=0
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

### 2.2 订阅 MQTT 消息

```console
    % ./bin/robust-ctl mqtt mqtt --server=127.0.0.1:1883 subscribe --username=admin --password=pwd123 --topic=test/topic1 --qos=0

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

### 2.3 发布保留消息

```console
    % ./bin/robust-ctl mqtt mqtt --server=127.0.0.1:1883 publish --username=admin --password=pwd123 --topic=\$share/group1/test/topic1 --qos=1 --retained
    able to connect: "127.0.0.1:1883"
    you can post a message on the terminal:
    helloworld!
    > You typed: helloworld!
    published retained message
```

```console
    % ./bin/robust-ctl mqtt mqtt --server=127.0.0.1:1883 subscribe --username=admin --password=pwd123 --topic=\$share/group1/test/topic1 --qos=0
    able to connect: "127.0.0.1:1883"
    subscribe success
    Retain message: helloworld!
```

## 3. 开启慢订阅功能

### 3.1 开启/关闭慢订阅

慢订阅统计功能主要是为了在消息到达 Broker 后，
Broker 来计算完成消息处理以及传输整个流程所消耗的时间(时延),
如果时延超过阈值，我们就会记录一条相关的信息在集群慢订阅日志当中，
运维人员可以通过命令查询整个集群下的慢订阅记录信息，
通过慢订阅信息来解决。

开启慢订阅

```console
% ./bin/robust-ctl mqtt mqtt slow-sub --enable=true
The slow subscription feature has been successfully enabled.
```

### 3.2 查询慢订阅记录

当我们启动了慢订阅统计功能之后, 集群就开启慢订阅统计功能，
这样我们可以通过对应的命令来去查询对应的慢订阅记录，
如果我们想要查看慢订阅记录，客户端可以输入如下命令

```console
% ./bin/robust-ctl mqtt mqtt slow-sub --query=true
+-----------+-------+----------+---------+-------------+
| client_id | topic | sub_name | time_ms | create_time |
+-----------+-------+----------+---------+-------------+
```

如果想要获取更多的慢订阅记录，
并且想要按照从小到大的顺序进行升序排序，
那么可以使用如下的命令

```console
% ./bin/robust-ctl mqtt mqtt slow-sub --list=200 --sort=asc
+-----------+-------+----------+---------+-------------+
| client_id | topic | sub_name | time_ms | create_time |
+-----------+-------+----------+---------+-------------+
```

对于慢订阅查询，我们同样支持筛选查询功能，我们支持使用 topic,
sub_name 以及 client_id 的方式来获取不同字段过滤后的结果，
其结果默认从大到小倒序排序，参考使用命令如下

```console
% ./bin/robust-ctl mqtt mqtt slow-sub --topic=topic_test1 --list=200
+-----------+-------+----------+---------+-------------+
| client_id | topic | sub_name | time_ms | create_time |
+-----------+-------+----------+---------+-------------+
```
