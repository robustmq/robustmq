# MQTT Multi-Tenancy

## What is Multi-Tenancy?

Multi-tenancy is a resource isolation mechanism provided by RobustMQ MQTT that allows multiple independent businesses or teams to share a single MQTT service while keeping their data, subscriptions, sessions, and permissions fully isolated from each other.

Each tenant has its own independent:

- Client connections and sessions
- Topic namespace
- Subscription management
- Access control (ACL) and blacklists
- Auto-subscription rules

## How to Specify a Tenant at Connect Time

Clients pass the tenant name during an MQTT CONNECT using one of the following three methods. The system resolves the tenant by priority from highest to lowest:

### Method 1: MQTT v5 User Property (Highest Priority)

Add the key-value pair `tenant = <tenant_name>` to the User Properties of the CONNECT packet.

```bash
# Connect to tenant "acme" using MQTTX CLI
mqttx conn \
  --mqtt-version 5 \
  -h 127.0.0.1 -p 1883 \
  -i device-001 \
  --user-properties "tenant: acme"
```

Use this when the client uses MQTT v5 and you want to keep the client_id and username free of tenant prefixes.

---

### Method 2: Username Prefix (Second Priority)

Embed the tenant name as a prefix in the username, separated by `@`: format is `<tenant>@<real_username>`.

```bash
# username "acme@alice" → tenant = acme, real username = alice
mqttx conn \
  -h 127.0.0.1 -p 1883 \
  -i device-001 \
  -u "acme@alice" \
  -P "password"
```

Use this when the client uses MQTT v3.1/v3.1.1 and can control the username field.

---

### Method 3: Client ID Prefix (Third Priority)

Embed the tenant name as a prefix in the client_id, separated by `@`: format is `<tenant>@<real_client_id>`.

```bash
# client_id "acme@device-001" → tenant = acme, real client_id = device-001
mqttx conn \
  -h 127.0.0.1 -p 1883 \
  -i "acme@device-001"
```

Use this when the client cannot modify the username but can control the client_id format.

---

### Method 4: Default Tenant (Fallback)

If none of the above methods carry tenant information, the connection is assigned to the system default tenant (`default`).

```bash
# No tenant information — uses the default tenant
mqttx conn \
  -h 127.0.0.1 -p 1883 \
  -i device-001
```

---

## Priority Summary

| Priority | Source | Example |
|----------|--------|---------|
| 1 (Highest) | MQTT v5 User Property | `tenant: acme` |
| 2 | Username prefix | `acme@alice` |
| 3 | Client ID prefix | `acme@device-001` |
| 4 (Fallback) | No tenant information | Uses `default` tenant |

When multiple sources are present at the same time, only the highest-priority source is used. For example, if both a User Property and a username prefix are set, the User Property takes precedence.

---

## Tenant Management

### Creating a Tenant

A tenant must be created before clients can connect to it. If a client specifies a tenant name that does not exist, the connection will be rejected.

To create a tenant via the RobustMQ Dashboard:

1. Visit [http://demo.robustmq.com/](http://demo.robustmq.com/)
2. Navigate to **System Management** → **Tenant Management**
3. Click **New Tenant**, enter the tenant name
4. Click **Confirm** to complete creation

### Viewing Tenant List

The **System Management** → **Tenant Management** page in the Dashboard lists all created tenants and their status.

---

## How Features Relate to Tenants

### Connections and Sessions

Each tenant's client connections and sessions are independent. Different tenants can have clients with the same client_id without conflict.

```bash
# device-001 under tenant "acme"
mqttx conn -i "acme@device-001" -h 127.0.0.1 -p 1883

# device-001 under tenant "beta" — completely independent
mqttx conn -i "beta@device-001" -h 127.0.0.1 -p 1883
```

### Topic Publish and Subscribe

Topics are isolated per tenant. Topics with the same name under different tenants do not interfere with each other:

```bash
# Tenant "acme" publishes to sensor/temp
mqttx pub -i "acme@pub-1" -u "acme@user" -t "sensor/temp" -m "25.3"

# Tenant "beta" subscribes to the same topic name — will NOT receive acme's messages
mqttx sub -i "beta@sub-1" -u "beta@user" -t "sensor/temp"
```

### Auto Subscription

Auto-subscription rules are configured per tenant and only apply to clients within that tenant. See [Auto Subscription](./AutoSubscription.md).

### Shared Subscription

Shared subscription groups are isolated within a tenant. Groups with the same name under different tenants are independent. See [Shared Subscription](./SharedSubscription.md).

### Access Control (ACL)

ACL rules and blacklists are configured and enforced per tenant. See the Security documentation for details.

---

## Notes

- **Tenant must exist first**: Ensure the target tenant has been created before connecting. Connections specifying a non-existent tenant will be rejected.
- **`@` is a reserved separator**: If the real client_id or username itself contains `@`, only the part before the first `@` is recognized as the tenant name; the rest is treated as the real value.
- **Default tenant**: Connections without any tenant information are assigned to the `default` tenant, which exists by default and does not need to be created manually.
- **MQTT v3 limitation**: MQTT v3.1/v3.1.1 does not support User Properties. Tenant information can only be passed via the username or client_id prefix.
