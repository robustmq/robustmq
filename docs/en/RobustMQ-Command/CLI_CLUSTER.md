# Cluster Commands

## Command Shape

```bash
robust-ctl cluster [--server <addr>] [--output table|json] <subcommand>
```

## Subcommands

### 1) status

Check cluster status.

```bash
robust-ctl cluster status
robust-ctl cluster --output json status
```

### 2) healthy

Check cluster healthy result.

```bash
robust-ctl cluster healthy
robust-ctl cluster --output json healthy
```

### 3) config

#### get

```bash
robust-ctl cluster config get
```

#### set

```bash
robust-ctl cluster config set \
  --config-type FlappingDetect \
  --config '{"enable":true}'
```

`config` is passed through to server-side config API as-is.

### 4) tenant

Manage cluster tenants (multi-tenancy support).

#### list

List all tenants.

```bash
robust-ctl cluster tenant list
robust-ctl cluster --output json tenant list
robust-ctl cluster --server 192.168.10.15:8080 tenant list
```

Sample output (table mode):

```
+-------------+-------------------+-------------+
| tenant_name | desc              | create_time |
+-------------+-------------------+-------------+
| business-a  | Business A tenant | 1738800000  |
| staging     | Staging env       | 1738900000  |
+-------------+-------------------+-------------+
```

#### create

Create a new tenant.

```bash
robust-ctl cluster tenant create -n <TENANT_NAME> [-d <DESC>]
```

| Flag | Short | Required | Description |
|------|-------|----------|-------------|
| `--tenant-name` | `-n` | Yes | Tenant name (1–128 chars) |
| `--desc` | `-d` | No | Description (max 500 chars) |

Examples:

```bash
# With description
robust-ctl cluster tenant create -n business-a -d "Business A tenant"

# Without description
robust-ctl cluster tenant create -n staging
```

#### delete

Delete a tenant.

```bash
robust-ctl cluster tenant delete -n <TENANT_NAME>
```

| Flag | Short | Required | Description |
|------|-------|----------|-------------|
| `--tenant-name` | `-n` | Yes | Name of the tenant to delete |

Example:

```bash
robust-ctl cluster tenant delete -n business-a
```

## Notes

- Tenants provide logical isolation within a single cluster. Suitable for serving multiple business units or multiple environments (dev / staging / prod) from one deployment.
- Use `--output json` for scripting and automation.
