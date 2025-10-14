# MQTT Broker Management Commands

## MQTT Broker Management (`mqtt`)

MQTT broker related operations, including session management, user management, ACL, blacklist, etc.

### Basic Syntax
```bash
robust-ctl mqtt [OPTIONS] <ACTION>
```

### Options
- `--server, -s <SERVER>`: Server address (default: 127.0.0.1:8080)

---

### 1.1 Session Management (`session`)

Manage MQTT sessions.

```bash
# List all sessions
robust-ctl mqtt session list
```

---

### 1.2 Subscription Management (`subscribes`)

Manage MQTT subscriptions.

```bash
# List all subscriptions
robust-ctl mqtt subscribes list
```

---

### 1.3 User Management (`user`)

Manage MQTT user accounts.

```bash
# List all users
robust-ctl mqtt user list

# Create user
robust-ctl mqtt user create \
  --username <USERNAME> \
  --password <PASSWORD> \
  [--is-superuser]

# Delete user
robust-ctl mqtt user delete --username <USERNAME>
```

**Parameter Description:**
- `--username, -u`: Username (required)
- `--password, -p`: Password (required)
- `--is-superuser, -i`: Whether it's a superuser (optional, default false)

---

### 1.4 Access Control List (`acl`)

Manage MQTT access control rules.

```bash
# List all ACL rules
robust-ctl mqtt acl list

# Create ACL rule
robust-ctl mqtt acl create \
  --cluster-name <CLUSTER_NAME> \
  --resource-type <RESOURCE_TYPE> \
  --resource-name <RESOURCE_NAME> \
  --topic <TOPIC> \
  --ip <IP> \
  --action <ACTION> \
  --permission <PERMISSION>

# Delete ACL rule
robust-ctl mqtt acl delete \
  --cluster-name <CLUSTER_NAME> \
  --resource-type <RESOURCE_TYPE> \
  --resource-name <RESOURCE_NAME> \
  --topic <TOPIC> \
  --ip <IP> \
  --action <ACTION> \
  --permission <PERMISSION>
```

**Parameter Description:**
- `--cluster-name, -c`: Cluster name (required)
- `--resource-type`: Resource type (ClientId, Username, IpAddress, etc.)
- `--resource-name`: Resource name (required)
- `--topic`: Topic (required)
- `--ip`: IP address (required)
- `--action`: Action type (All, Publish, Subscribe, PubSub)
- `--permission`: Permission (Allow, Deny)

---

### 1.5 Blacklist Management (`blacklist`)

Manage MQTT blacklist.

```bash
# List all blacklist entries
robust-ctl mqtt blacklist list

# Create blacklist entry
robust-ctl mqtt blacklist create \
  --cluster-name <CLUSTER_NAME> \
  --blacklist-type <BLACKLIST_TYPE> \
  --resource-name <RESOURCE_NAME> \
  --end-time <END_TIME> \
  --desc <DESCRIPTION>

# Delete blacklist entry
robust-ctl mqtt blacklist delete \
  --cluster-name <CLUSTER_NAME> \
  --blacklist-type <BLACKLIST_TYPE> \
  --resource-name <RESOURCE_NAME>
```

**Parameter Description:**
- `--cluster-name, -c`: Cluster name (required)
- `--blacklist-type`: Blacklist type (ClientId, IpAddress, Username)
- `--resource-name, -r`: Resource name (required)
- `--end-time`: End time (Unix timestamp) (required)
- `--desc`: Description (required)

---

### 1.6 Client Connection Management (`client`)

Manage MQTT client connections.

```bash
# List all client connections
robust-ctl mqtt client list
```

---

### 1.7 Topic Management (`topic`)

Manage MQTT topics.

```bash
# List all topics
robust-ctl mqtt topic list
```

---

### 1.8 Topic Rewrite Rules (`topic-rewrite`)

Manage topic rewrite rules.

```bash
# List all topic rewrite rules
robust-ctl mqtt topic-rewrite list

# Create topic rewrite rule
robust-ctl mqtt topic-rewrite create \
  --action <ACTION> \
  --source-topic <SOURCE_TOPIC> \
  --dest-topic <DEST_TOPIC> \
  --regex <REGEX>

# Delete topic rewrite rule
robust-ctl mqtt topic-rewrite delete \
  --action <ACTION> \
  --source-topic <SOURCE_TOPIC>
```

**Parameter Description:**
- `--action, -a`: Action type (required)
- `--source-topic, -s`: Source topic (required)
- `--dest-topic, -d`: Destination topic (required for creation)
- `--regex, -r`: Regular expression (required for creation)

---

### 1.9 Connector Management (`connector`)

Manage MQTT connectors.

```bash
# List all connectors
robust-ctl mqtt connector list --connector-name <NAME>

# Create connector
robust-ctl mqtt connector create \
  --connector-name <CONNECTOR_NAME> \
  --connector-type <CONNECTOR_TYPE> \
  --config <CONFIG> \
  --topic-id <topic_name>

# Delete connector
robust-ctl mqtt connector delete --connector-name <CONNECTOR_NAME>
```

**Parameter Description:**
- `--connector-name, -c`: Connector name (required)
- `--connector-type, -c`: Connector type (required for creation)
- `--config, -c`: Configuration information (required for creation)
- `--topic-id, -t`: Topic ID (required for creation)

---

### 1.10 Schema Management (`schema`)

Manage MQTT message schemas.

```bash
# List all schemas
robust-ctl mqtt schema list --schema-name <NAME>

# Create schema
robust-ctl mqtt schema create \
  --schema-name <SCHEMA_NAME> \
  --schema-type <SCHEMA_TYPE> \
  --schema <SCHEMA> \
  --desc <DESCRIPTION>

# Delete schema
robust-ctl mqtt schema delete --schema-name <SCHEMA_NAME>

# List schema bindings
robust-ctl mqtt schema list-bind \
  --schema-name <SCHEMA_NAME> \
  --resource-name <RESOURCE_NAME>

# Bind schema
robust-ctl mqtt schema bind \
  --schema-name <SCHEMA_NAME> \
  --resource-name <RESOURCE_NAME>

# Unbind schema
robust-ctl mqtt schema unbind \
  --schema-name <SCHEMA_NAME> \
  --resource-name <RESOURCE_NAME>
```

**Parameter Description:**
- `--schema-name, -s`: Schema name (required)
- `--schema-type, -t`: Schema type (required for creation)
- `--schema, -s`: Schema definition (required for creation)
- `--desc, -d`: Description (required for creation)
- `--resource-name, -r`: Resource name (required for binding operations)

---

### 1.11 Auto Subscribe Rules (`auto-subscribe`)

Manage auto subscribe rules.

```bash
# List all auto subscribe rules
robust-ctl mqtt auto-subscribe list

# Create auto subscribe rule
robust-ctl mqtt auto-subscribe create \
  --topic <TOPIC> \
  [--qos <QOS>] \
  [--no-local] \
  [--retain-as-published] \
  [--retained-handling <HANDLING>]

# Delete auto subscribe rule
robust-ctl mqtt auto-subscribe delete --topic <TOPIC>
```

**Parameter Description:**
- `--topic, -t`: Topic (required)
- `--qos, -q`: QoS level (optional, default 0)
- `--no-local, -n`: Don't receive local messages (optional, default false)
- `--retain-as-published, -r`: Retain as published (optional, default false)
- `--retained-handling, -R`: Retained message handling (optional, default 0)

---

### 1.12 Publish Messages (`publish`)

Publish MQTT messages.

```bash
# Publish message (interactive mode)
robust-ctl mqtt publish \
  --username <USERNAME> \
  --password <PASSWORD> \
  --topic <TOPIC> \
  [--qos <QOS>] \
  [--retained]
```

**Parameter Description:**
- `--username, -u`: Username (required)
- `--password, -p`: Password (required)
- `--topic, -t`: Topic (required)
- `--qos, -q`: QoS level (optional, default 0)
- `--retained`: Retained message (optional, default false)

---

### 1.13 Subscribe Messages (`subscribe`)

Subscribe to MQTT messages.

```bash
# Subscribe to messages
robust-ctl mqtt subscribe \
  --username <USERNAME> \
  --password <PASSWORD> \
  --topic <TOPIC> \
  [--qos <QOS>]
```

**Parameter Description:**
- `--username, -u`: Username (required)
- `--password, -p`: Password (required)
- `--topic, -t`: Topic (required)
- `--qos, -q`: QoS level (optional, default 0)

---

### 1.14 Observability Features

#### Slow Subscribe Monitoring (`slow-subscribe`)
```bash
# List slow subscriptions
robust-ctl mqtt slow-subscribe list
```

#### Flapping Detection (`flapping-detect`)
```bash
# List flapping detection results
robust-ctl mqtt flapping-detect
```

#### System Alarms (`system-alarm`)
```bash
# List system alarms
robust-ctl mqtt system-alarm list
```
