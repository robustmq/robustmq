# Authorization

Access Control Lists (ACL) are an important component of MQTT security mechanisms. RobustMQ provides a flexible and fine-grained permission control system that can perform access control based on multiple dimensions such as user, client ID, topic, and IP address.

## Overview

RobustMQ's ACL authorization system has the following features:

- **Fine-grained Permission Control**: Supports permission control for different operations such as publish, subscribe, and retained messages
- **Multi-dimensional Matching**: Supports permission matching based on username, client ID, topic, and IP address
- **Flexible Permission Policies**: Supports both Allow and Deny permission types
- **Wildcard Support**: Supports using wildcards for topic and IP address matching
- **Super User Bypass**: Super users can bypass all ACL checks
- **High-performance Caching**: ACL rules are cached in memory to ensure high-performance access control

## Permission Check Flow

RobustMQ's permission checks proceed in the following order:

1. **Super User Check**: If it's a super user, allow all operations directly
2. **Blacklist Check**: Check if the user, client ID, or IP is in the blacklist
3. **ACL Rule Check**: Check ACL rules by priority, deny access if a matching deny rule is found
4. **Retained Message Permission Check**: If it's a retained message, additionally check Retain permissions
5. **Default Policy**: If no matching deny rules are found, allow access

## ACL Permission Types

### Operation Types

| Operation Type | Value | Description                                       |
| -------------- | ----- | ------------------------------------------------- |
| All            | 0     | All operations (publish, subscribe, retain, etc.) |
| Subscribe      | 1     | Subscribe to topics                               |
| Publish        | 2     | Publish messages                                  |
| PubSub         | 3     | Publish and subscribe                             |
| Retain         | 4     | Retained messages                                 |
| Qos            | 5     | QoS related operations                            |

### Permission Types

| Permission Type | Value | Description  |
| --------------- | ----- | ------------ |
| Allow           | 1     | Allow access |
| Deny            | 0     | Deny access  |

### Resource Types

| Resource Type | Description                           |
| ------------- | ------------------------------------- |
| User          | Permission control based on username  |
| ClientId      | Permission control based on client ID |

## Configure ACL Rules

### Using Command Line Tool

#### Create ACL Rules

```bash
# Allow user testuser to publish to test/# topic
robust-ctl mqtt acl create \
  --cluster-name robustmq-cluster \
  --resource-type User \
  --resource-name testuser \
  --topic "test/#" \
  --ip "*" \
  --action Publish \
  --permission Allow

# Deny client bad_client from subscribing to all topics
robust-ctl mqtt acl create \
  --cluster-name robustmq-cluster \
  --resource-type ClientId \
  --resource-name bad_client \
  --topic "*" \
  --ip "*" \
  --action Subscribe \
  --permission Deny

# Allow users from specific IP address to access all topics
robust-ctl mqtt acl create \
  --cluster-name robustmq-cluster \
  --resource-type User \
  --resource-name admin \
  --topic "*" \
  --ip "192.168.1.100" \
  --action All \
  --permission Allow
```

#### Query ACL Rules

```bash
# Query all ACL rules
robust-ctl mqtt acl list
```

#### Delete ACL Rules

```bash
# Delete specific ACL rule
robust-ctl mqtt acl delete \
  --cluster-name robustmq-cluster \
  --resource-type User \
  --resource-name testuser \
  --topic "test/#" \
  --ip "*" \
  --action "Subscribe" \
  --permission "Allow"
```

### Using HTTP API

#### Create ACL Rules

```bash
curl -X POST http://localhost:8080/api/mqtt/acl/create \
  -H "Content-Type: application/json" \
  -d '{
    "resource_type": "User",
    "resource_name": "testuser",
    "topic": "test/#",
    "ip": "*",
    "action": "Publish",
    "permission": "Allow"
  }'
```

#### Query ACL Rules

```bash
# Query all ACL rules
curl -X POST http://localhost:8080/api/mqtt/acl/list \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 10,
    "page": 1
  }'

# Query ACL rules for specific user (using filter parameters)
curl -X POST http://localhost:8080/api/mqtt/acl/list \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 10,
    "page": 1,
    "filter_field": "resource_name",
    "filter_values": ["testuser"],
    "exact_match": "true"
  }'
```

#### Delete ACL Rules

```bash
curl -X POST http://localhost:8080/api/mqtt/acl/delete \
  -H "Content-Type: application/json" \
  -d '{
    "resource_type": "User",
    "resource_name": "testuser",
    "topic": "test/#",
    "ip": "*",
    "action": "Publish",
    "permission": "Allow"
  }'
```

### Direct Database Operations

If using MySQL storage backend, you can operate the database directly:

```sql
-- Create ACL rule
INSERT INTO mqtt_acl (allow, ipaddr, username, clientid, access, topic)
VALUES (1, '*', 'testuser', '', 2, 'test/#');

-- Query ACL rules
SELECT * FROM mqtt_acl WHERE username = 'testuser';

-- Delete ACL rule
DELETE FROM mqtt_acl WHERE username = 'testuser' AND topic = 'test/#';
```

## Troubleshooting

### Permission Denied

1. **Check ACL Rules**: Confirm if there are matching deny rules
2. **Check Super User Status**: Confirm if the user is a super user
3. **Check Blacklist**: Confirm if the user, client ID, or IP is in the blacklist
4. **View Logs**: Check RobustMQ logs for detailed permission check information

### Permission Configuration Not Taking Effect

1. **Cache Refresh**: Manually refresh ACL cache
2. **Configuration Sync**: Confirm configuration has been synchronized to all nodes
3. **Rule Priority**: Check if there are higher priority rules overriding
4. **Wildcard Matching**: Confirm wildcard usage is correct

### Performance Issues

1. **Optimize Rule Count**: Reduce unnecessary ACL rules
2. **Use More Precise Matching**: Avoid overusing wildcards
3. **Monitor Cache Status**: Check ACL cache hit rate
4. **Database Optimization**: Optimize indexes on ACL data tables

## Common Questions

### Q: What is the priority of ACL rules?

A: RobustMQ's ACL checks follow the following priority:

1. Super users bypass all checks
2. Blacklist checks take priority over ACL
3. Deny rules take priority over allow rules
4. User-level rules and client ID-level rules have equal priority

### Q: How to implement topic-level permission inheritance?

A: Use wildcards to implement permission inheritance:

```bash
# Allow access to all sub-topics under sensors
robust-ctl mqtt acl create \
  --resource-type user \
  --resource-name sensor_user \
  --topic "sensors/#" \
  --action subscribe \
  --permission allow \
  --ip "*"
```

### Q: How to debug permission issues?

A: Enable detailed logging and check the permission check process:

```toml
[log]
level = "debug"
```

Then view the permission check information in the logs.

### Q: Do ACL rules support regular expressions?

A: Currently, ACL rule topic matching uses MQTT standard wildcards (+ and #) and does not support regular expressions. IP addresses support CIDR format.

### Q: How to batch import ACL rules?

A: You can use scripts to create rules in batches, or operate the database directly for batch import:

```bash
#!/bin/bash
while IFS=',' read -r resource_type resource_name topic action permission ip
do
    robust-ctl mqtt acl create \
      --resource-type "$resource_type" \
      --resource-name "$resource_name" \
      --topic "$topic" \
      --action "$action" \
      --permission "$permission" \
      --ip "$ip"
done < acl_rules.csv
```
