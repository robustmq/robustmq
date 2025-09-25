# Blacklist

Blacklist is an important component of RobustMQ's security mechanism, used to block malicious clients or suspicious IP addresses from connecting and accessing. The blacklist feature provides a flexible restriction mechanism that supports multiple matching patterns and time limits.

## Overview

RobustMQ's blacklist system has the following features:

- **Multi-dimensional Restrictions**: Supports blacklist control based on username, client ID, and IP address
- **Pattern Matching**: Supports exact matching, regular expression matching, and CIDR network segment matching
- **Time Limits**: Supports setting expiration time for blacklists, automatically lifting restrictions
- **High Priority**: Blacklist checks take priority over ACL permission checks
- **High-performance Caching**: Blacklist rules are cached in memory to ensure fast access control
- **Dynamic Management**: Supports dynamic addition, deletion, and updating of blacklist rules

## Blacklist Check Flow

RobustMQ's blacklist checks occur in the early stages of permission verification, in the following order:

1. **User Blacklist Check**: Check if the username is in the blacklist
2. **User Pattern Matching Check**: Use regular expressions to match usernames
3. **Client ID Blacklist Check**: Check if the client ID is in the blacklist
4. **Client ID Pattern Matching Check**: Use regular expressions to match client IDs
5. **IP Address Blacklist Check**: Check if the IP address is in the blacklist
6. **IP Network Segment Matching Check**: Use CIDR format to match IP address ranges

## Blacklist Types

### Basic Types

| Type     | Description          | Matching Method |
| -------- | -------------------- | --------------- |
| User     | Username blacklist   | Exact match     |
| ClientId | Client ID blacklist  | Exact match     |
| IP       | IP address blacklist | Exact match     |

### Advanced Types

| Type          | Description                 | Matching Method    |
| ------------- | --------------------------- | ------------------ |
| UserMatch     | Username pattern matching   | Regular expression |
| ClientIdMatch | Client ID pattern matching  | Regular expression |
| IPCIDR        | IP network segment matching | CIDR format        |

## Configure Blacklist

### Using Command Line Tool

#### Add Blacklist

```bash
# Add user blacklist
robust-ctl mqtt blacklist create \
  --cluster-name robustmq-cluster \
  --blacklist-type User \
  --resource-name malicious_user \
  --end-time "2024-12-31 23:59:59" \
  --desc "Malicious user, violating terms of use"

# Add client ID blacklist
robust-ctl mqtt blacklist create \
  --cluster-name robustmq-cluster \
  --blacklist-type ClientId \
  --resource-name bad_client_001 \
  --end-time "2024-06-30 12:00:00" \
  --desc "Abnormal client, frequently sending spam messages"

# Add IP address blacklist
robust-ctl mqtt blacklist create \
  --cluster-name robustmq-cluster \
  --blacklist-type Ip \
  --resource-name "192.168.1.100" \
  --end-time "2024-03-31 00:00:00" \
  --desc "Suspicious IP, attempting brute force attacks"

# Add username pattern matching blacklist
robust-ctl mqtt blacklist create \
  --cluster-name robustmq-cluster \
  --blacklist-type UserMatch \
  --resource-name "test.*" \
  --end-time "2024-12-31 23:59:59" \
  --desc "Test users prohibited from accessing production environment"

# Add client ID pattern matching blacklist
robust-ctl mqtt blacklist create \
  --cluster-name robustmq-cluster \
  --blacklist-type ClientIdMatch \
  --resource-name "bot_.*" \
  --end-time "2024-12-31 23:59:59" \
  --desc "Prohibit bot clients"

# Add IP network segment blacklist
robust-ctl mqtt blacklist create \
  --cluster-name robustmq-cluster \
  --blacklist-type IPCIDR \
  --resource-name "10.0.0.0/24" \
  --end-time "2024-12-31 23:59:59" \
  --desc "Prohibit specific network segment access"
```

#### Query Blacklist

```bash
# Query all blacklists
robust-ctl mqtt blacklist list
```

#### Delete Blacklist

```bash
# Delete specific blacklist
robust-ctl mqtt blacklist delete \
  --cluster-name robustmq-cluster \
  --blacklist-type User \
  --resource-name malicious_user
```

### Using HTTP API

#### Add Blacklist

```bash
curl -X POST http://localhost:8080/api/mqtt/blacklist/create \
  -H "Content-Type: application/json" \
  -d '{
    "blacklist_type": "User",
    "resource_name": "malicious_user",
    "end_time": 1735689599,
    "desc": "Malicious user, violating terms of use"
  }'
```

#### Query Blacklist

```bash
# Query all blacklists
curl -X POST http://localhost:8080/api/mqtt/blacklist/list \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 10,
    "page": 1
  }'

# Query specific type blacklist (using filter parameters)
curl -X POST http://localhost:8080/api/mqtt/blacklist/list \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 10,
    "page": 1,
    "filter_field": "blacklist_type",
    "filter_values": ["User"],
    "exact_match": "true"
  }'
```

#### Delete Blacklist

```bash
curl -X POST http://localhost:8080/api/mqtt/blacklist/delete \
  -H "Content-Type: application/json" \
  -d '{
    "blacklist_type": "User",
    "resource_name": "malicious_user"
  }'
```

### Direct Storage Operations

If using Placement Center storage, you can operate through internal APIs:

```rust
// Add blacklist
let blacklist = MqttAclBlackList {
    blacklist_type: MqttAclBlackListType::User,
    resource_name: "malicious_user".to_string(),
    end_time: now_second() + 86400, // Expires after 24 hours
    desc: "Malicious user".to_string(),
};
cache_manager.add_blacklist(blacklist);
```

## Performance Optimization

### Blacklist Caching

RobustMQ uses an efficient memory caching mechanism:

- Exact match blacklists use hash tables with O(1) lookup time
- Pattern match blacklists are cached in groups to reduce regular expression computation
- Expiration checks use time indexing for fast cleanup of expired rules

### Performance Monitoring

```bash
# Monitor blacklist performance metrics
curl http://localhost:8080/metrics | grep -E "blacklist_(check|cache)"

# Example metrics:
# mqtt_blacklist_check_duration_seconds{type="user"} 0.001
# mqtt_blacklist_cache_hits_total 1024
# mqtt_blacklist_cache_misses_total 12
```

### Optimization Recommendations

1. **Reduce Regular Expression Complexity**: Avoid using overly complex regular expressions
2. **Set Reasonable Expiration Times**: Clean up expired rules promptly to reduce memory usage
3. **Batch Operations**: Use batch APIs for large-scale blacklist operations
4. **Monitor Hit Rate**: Regularly check blacklist hit rates and clean up invalid rules

## Troubleshooting

### Connection Blocked by Blacklist

1. **Check Blacklist Rules**: Confirm if the client matches blacklist rules
2. **Check Time Settings**: Confirm if the blacklist has expired
3. **Verify Matching Patterns**: Check if regular expressions or CIDR format are correct
4. **View Detailed Logs**: Enable debug logs to view detailed matching process

### Blacklist Not Taking Effect

1. **Cache Refresh**: Manually refresh blacklist cache
2. **Configuration Sync**: Confirm configuration has been synchronized to all nodes
3. **Rule Format Check**: Verify regular expressions or CIDR format
4. **Time Check**: Confirm blacklist has not expired

### Performance Issues

1. **Optimize Regular Expressions**: Simplify complex matching patterns
2. **Clean Expired Rules**: Regularly clean up expired blacklists
3. **Monitor Memory Usage**: Check memory usage of blacklist cache
4. **Batch Operation Optimization**: Use batch APIs to reduce operation frequency

## Common Questions

### Q: What is the priority of blacklists?

A: Blacklist checks take priority over all other permission checks, including super user privileges. Even super users will be denied access if they are in the blacklist.

### Q: How is the performance of regular expression matching?

A: RobustMQ has optimized regular expression matching using compilation caching and grouped matching. It is recommended to avoid overly complex regular expressions for best performance.

### Q: How to implement temporary blacklists?

A: Set an appropriate expiration time to implement temporary blacklists:

```bash
# 1-hour temporary blacklist
robust-ctl mqtt blacklist create \
  --cluster-name robustmq-cluster \
  --blacklist-type User \
  --resource-name temp_user \
  --end-time "$(date -d '+1 hour' '+%Y-%m-%d %H:%M:%S')" \
  --desc "Temporary restriction for 1 hour"
```

### Q: Do blacklists support wildcards?

A: Exact match types (User, ClientId, IP) do not support wildcards, but you can use pattern match types (UserMatch, ClientIdMatch, IPCIDR) to achieve similar functionality.

### Q: How to batch clean expired blacklists?

A: You can use management APIs or scheduled tasks for cleanup:

```bash
# Script to clean expired blacklists
#!/bin/bash
current_time=$(date +%s)
robust-ctl mqtt blacklist list | \
jq -r ".[] | select(.end_time < $current_time) | \"\\(.blacklist_type) \\(.resource_name)\"" | \
while read blacklist_type resource_name; do
    robust-ctl mqtt blacklist delete \
      --cluster-name robustmq-cluster \
      --blacklist-type "$blacklist_type" \
      --resource-name "$resource_name"
done
```

### Q: Are blacklist rules automatically synchronized to the cluster?

A: Blacklist rules are synchronized to all nodes in the cluster through a periodic cache synchronization mechanism. Unlike user and ACL rules, blacklists do not support real-time push synchronization, but rely on each broker node pulling the latest blacklist data from meta-service every second to update the local cache.
