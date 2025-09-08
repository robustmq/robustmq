# Cluster Management Commands

## Cluster Management (`cluster`)

Cluster configuration management.

### Basic Syntax
```bash
robust-ctl cluster [OPTIONS] <ACTION>
```

### Options
- `--server, -s <SERVER>`: Server address (default: 127.0.0.1:8080)

### Configuration Management (`config`)

```bash
# Get cluster configuration
robust-ctl cluster config get
```

---

## Usage Examples

```bash
# View cluster configuration help
robust-ctl cluster --help
robust-ctl cluster config --help

# Connect to specified server to get configuration
robust-ctl cluster --server 192.168.1.100:8080 config get

# Get local cluster configuration
robust-ctl cluster config get
```

---

## Functionality

The cluster management module is mainly used for:

1. **Configuration Viewing**: Get current cluster configuration information
2. **Cluster Status**: Understand the cluster's running status
3. **Configuration Management**: View and manage cluster-level configuration parameters

---

## Notes

- Cluster configuration operations usually require administrator privileges
- Ensure network connection to the correct cluster management service
- Configuration information is returned in JSON format for easy parsing and processing
