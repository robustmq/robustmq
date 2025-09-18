# RobustMQ CLI Common Guide

## Overview

`robust-ctl` is the command-line management tool for RobustMQ, built with Rust clap library, providing management capabilities for MQTT broker, cluster configuration, and journal engine.

## Installation and Build

```bash
# Build the project
cargo build --release

# Run the tool
./target/release/robust-ctl --help
```

## Basic Syntax

```bash
robust-ctl [OPTIONS] <COMMAND>
```

## Global Options

- `--help, -h`: Show help information
- `--version, -V`: Show version information

---

## Documentation Navigation

- 🔧 **[MQTT Broker Management](CLI_MQTT.md)** - All MQTT broker related commands
- 🏗️ **[Cluster Management](CLI_CLUSTER.md)** - Cluster configuration management commands
- 📝 **[Journal Engine Management](CLI_JOURNAL.md)** - Journal engine related commands

## Quick Command Index

### MQTT Broker Management Commands
```bash
robust-ctl mqtt [OPTIONS] <ACTION>
```

Main Features:
- Session Management (`session`)
- Subscription Management (`subscribes`)
- User Management (`user`)
- Access Control List (`acl`)
- Blacklist Management (`blacklist`)
- Client Connection Management (`client`)
- Topic Management (`topic`)
- Topic Rewrite Rules (`topic-rewrite`)
- Connector Management (`connector`)
- Schema Management (`schema`)
- Auto Subscribe Rules (`auto-subscribe`)
- Publish Messages (`publish`)
- Subscribe Messages (`subscribe`)
- Observability Features

### Cluster Management Commands
```bash
robust-ctl cluster [OPTIONS] <ACTION>
```

Main Features:
- Configuration Management (`config`)

### Journal Engine Management Commands
```bash
robust-ctl journal [OPTIONS]
```

Main Features:
- Status Viewing (currently under development)

## Quick Start

```bash
# View tool help
robust-ctl --help

# View MQTT command help
robust-ctl mqtt --help

# List all users
robust-ctl mqtt user list

# Get cluster configuration
robust-ctl cluster config get
```

## Getting Help

Each command supports the `--help` parameter to get detailed help:

```bash
robust-ctl --help                    # General tool help
robust-ctl mqtt --help               # MQTT module help
robust-ctl mqtt user --help          # User management help
robust-ctl cluster --help            # Cluster management help
robust-ctl journal --help            # Journal engine help
```

---

## Usage Examples

### Basic Operation Examples

```bash
# 1. View help
robust-ctl --help
robust-ctl mqtt --help
robust-ctl mqtt user --help

# 2. Connect to specified server
robust-ctl mqtt --server 192.168.1.100:8080 session list

# 3. User management
robust-ctl mqtt user create --username testuser --password testpass
robust-ctl mqtt user list
robust-ctl mqtt user delete --username testuser

# 4. ACL management
robust-ctl mqtt acl create \
  --cluster-name mycluster \
  --resource-type ClientId \
  --resource-name client001 \
  --topic "sensor/+" \
  --ip "192.168.1.0/24" \
  --action Publish \
  --permission Allow

# 5. Publish and subscribe
robust-ctl mqtt publish \
  --username testuser \
  --password testpass \
  --topic "test/topic" \
  --qos 1

robust-ctl mqtt subscribe \
  --username testuser \
  --password testpass \
  --topic "test/topic" \
  --qos 1

# 6. Cluster configuration
robust-ctl cluster config get
```

### Advanced Management Examples

```bash
# 1. Blacklist management
robust-ctl mqtt blacklist create \
  --cluster-name mycluster \
  --blacklist-type ClientId \
  --resource-name malicious_client \
  --end-time 1735689600 \
  --desc "Blocked due to suspicious activity"

# 2. Topic rewrite rules
robust-ctl mqtt topic-rewrite create \
  --action redirect \
  --source-topic "old/topic/+" \
  --dest-topic "new/topic/+" \
  --regex "old/(.*)"

# 3. Auto subscribe rules
robust-ctl mqtt auto-subscribe create \
  --topic "system/alerts/+" \
  --qos 2 \
  --no-local \
  --retain-as-published

# 4. Schema management
robust-ctl mqtt schema create \
  --schema-name temperature_schema \
  --schema-type json \
  --schema '{"type":"object","properties":{"temp":{"type":"number"}}}' \
  --desc "Temperature sensor data schema"
```

---

## Error Handling

The tool provides detailed error information and help:

- Use `--help` or `-h` to get command help
- Check server connection status
- Validate parameter format and permissions
- View detailed error messages and suggestions

---

## Configuration

The tool connects to `127.0.0.1:8080` by default. You can specify a different server address using the `--server` parameter.

---

## Notes

1. **Permission Requirements**: Some operations require administrator privileges
2. **Network Connection**: Ensure access to RobustMQ server
3. **Parameter Validation**: The tool validates parameter format and validity
4. **Interactive Mode**: Publish and subscribe commands support interactive mode
5. **Pagination Support**: List commands support paginated display (default 10000 items per page)

---

## Version Information

- Tool Version: 0.0.1
- Author: RobustMQ Team
- Based on: Rust clap library

---

## More Information

For more detailed information, please refer to [RobustMQ Official Documentation](https://robustmq.com).
