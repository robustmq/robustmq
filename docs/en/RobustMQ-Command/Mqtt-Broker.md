# MQTT Broker Command

## 1. User Management

MQTT Broker has enabled user authentication. Clients must provide valid usernames and passwords before publishing or subscribing to messages to pass the authentication. Clients that fail authentication will not be able to communicate with the Broker. This feature enhances system security and prevents unauthorized access.

### 1.1 Create User

Create a new MQTT Broker user.

```console
% ./bin/robust-ctl mqtt mqtt user create --username=testp --password=7355608 --is_superuser=false
Created successfully!
```

### 1.2 Delete User

```console
% ./bin/robust-ctl mqtt mqtt user delete --username=testp
Deleted successfully!
```

### 1.3 List User

```console
% ./bin/robust-ctl mqtt mqtt user list
+----------+--------------+
| username | is_superuser |
+----------+--------------+
| admin    | true         |
+----------+--------------+
| testp    | false        |
+----------+--------------+
```

## 2. Pub & Sub

### 2.1 publish

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

### 2.2 subscribe

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

### 2.3 Pub retain message

```console
% ./bin/robust-ctl mqtt mqtt --server=127.0.0.1:1883 publish --username=admin --password=pwd123 --topic=\$share/group1/test/topic1 --qos=1 --retained
able to connect: "127.0.0.1:1883"
you can post a message on the terminal:
helloworld!
> You typed: helloworld!
published retained message
```

## 3. ACL (Access Control List) Management

### 3.1 Create ACL

Create a new ACL rule.

```console
% ./bin/robust-ctl mqtt mqtt --server=127.0.0.1:1883 acl create --cluster-name=admin --acl=xxx
able to connect: "127.0.0.1:1883"
Created successfully!
```

### 3.2 Delete ACL

Delete an existing ACL rule.

```console
% ./bin/robust-ctl mqtt mqtt --server=127.0.0.1:1883 acl delete --cluster-name=admin --acl=xxx
able to connect: "127.0.0.1:1883"
Deleted successfully!
```

### 3.3 ACL List

List all created ACL rules.

```console
% ./bin/robust-ctl mqtt mqtt --server=127.0.0.1:1883 acl list
+---------------+---------------+-------+----+--------+------------+
| resource_type | resource_name | topic | ip | action | permission |
+---------------+---------------+-------+----+--------+------------+
```

## 4. Blacklist Management

### 4.1 Create Blacklist

Create a new blacklist rule.

```console
% ./bin/robust-ctl mqtt mqtt --server=127.0.0.1:1883 blacklist create --cluster-name=admin --blacklist=client_id
able to connect: "127.0.0.1:1883"
Created successfully!
```

### 4.2 Delete Blacklist

Delete an existing blacklist rule.

```console
% ./bin/robust-ctl mqtt mqtt --server=127.0.0.1:1883 blacklist delete --cluster-name=admin --blacklist-type=client_id --resource-name=client1
able to connect: "127.0.0.1:1883"
Deleted successfully!
```

### 4.3 Blacklist List

List all created blacklist rules.

```console
```console
% ./bin/robust-ctl mqtt mqtt --server=127.0.0.1:1883 blacklist list
+----------------+---------------+----------+----------------+
| blacklist_type | resource_name | end_time | blacklist_type |
+----------------+---------------+----------+----------------+
```

## 5. Slow Subscription

The slow subscription statistics function is mainly used to calculate the time (latency) it takes for the Broker to complete message processing and transmission after the message arrives at the Broker. If the latency exceeds the threshold, we will record a related piece of information in the cluster's slow subscription log. Operations personnel can query slow subscription records across the entire cluster using commands to address issues based on this information

### 5.1 Enable Slow Subscription

- Enable slow subscription

```console
% ./bin/robust-ctl mqtt mqtt slow-sub --enable=true
The slow subscription feature has been successfully enabled.
```

- Close slow subscription

```console
% ./bin/robust-ctl mqtt mqtt slow-sub --enable=false
The slow subscription feature has been successfully closed.
```

### 5.2 View slow subscription records

After enabling the slow subscription statistics function, the cluster begins recording slow subscriptions. To query corresponding slow subscription records, clients can enter the following command:

```console
% ./bin/robust-ctl mqtt mqtt slow-sub --query=true
+-----------+-------+----------+---------+-------------+
| client_id | topic | sub_name | time_ms | create_time |
+-----------+-------+----------+---------+-------------+
```

### 5.3 Sorting Functionality

To obtain more slow subscription records and sort them in ascending order from smallest to largest, you can use the following command:

```console
% ./bin/robust-ctl mqtt mqtt slow-sub --list=200 --sort=asc
+-----------+-------+----------+---------+-------------+
| client_id | topic | sub_name | time_ms | create_time |
+-----------+-------+----------+---------+-------------+
```

### 5.4 Filtering Query Functionality

For slow subscription queries, filtering queries are also supported. You can retrieve filtered results by fields such as topic, sub_name, and client_id. By default, the results are sorted in descending order from largest to smallest. Refer to the usage command below:

```console
% ./bin/robust-ctl mqtt mqtt slow-sub --topic=topic_test1 --list=200
+-----------+-------+----------+---------+-------------+
| client_id | topic | sub_name | time_ms | create_time |
+-----------+-------+----------+---------+-------------+
```
