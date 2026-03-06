# MongoDB Data Source

MongoDB data source is for environments where user/ACL/blacklist data is already managed in MongoDB.

## Suitable Scenarios

- Existing identity/policy services already store data in MongoDB.
- You want collection + filter based integration without schema migration.
- You need broker cache-first auth behavior.

## Core Capabilities

- Sync user/ACL/blacklist data from MongoDB.
- Configure dedicated collections via `collection_user/collection_acl/collection_blacklist`.
- Configure dedicated JSON filters via `query_user/query_acl/query_blacklist`.

## Runtime Model (Brief)

1. Broker periodically reads MongoDB data.
2. Parsed data is written into local cache.
3. CONNECT and access checks use cache first (no per-request MongoDB query).

## Configuration

Key fields in `mongodb_config`:

- `mongodb_uri`: MongoDB connection URI
- `database`: database name
- `username` / `password`: credentials (optional, can be embedded in URI)
- `collection_user`: user collection
- `collection_acl`: ACL collection
- `collection_blacklist`: blacklist collection
- `query_user`: JSON filter string for users
- `query_acl`: JSON filter string for ACLs
- `query_blacklist`: JSON filter string for blacklist

## Field Contract

### User document

Required:

- `username`
- `password`

Optional:

- `salt`
- `is_superuser` (`bool`, `0|1`, or `"true"|"false"`)
- `created` (DateTime, unix timestamp, or parseable datetime string)

### ACL document

Required:

- `permission` (`1|0` or `Allow|Deny`)
- `action` or `access` (`0..5` or `All|Subscribe|Publish|PubSub|Retain|Qos`)
- `topic` or `topics` (array)

Optional:

- `username`
- `clientid`
- `ipaddr` (or `ipaddress`)

### Blacklist document

Required:

- `blacklist_type` (`ClientId` / `User` / `Ip` / `ClientIdMatch` / `UserMatch` / `IPCIDR`)
- `resource_name`
- `end_time`

Optional:

- `desc`

## Example

```toml
[[mqtt.auth]]
authn_type = "password_based"

[mqtt.auth.config.storage_config]
storage_type = "mongodb"

[mqtt.auth.config.storage_config.mongodb_config]
mongodb_uri = "mongodb://127.0.0.1:27017"
database = "mqtt"
username = ""
password = ""
collection_user = "mqtt_user"
collection_acl = "mqtt_acl"
collection_blacklist = "mqtt_blacklist"
query_user = "{}"
query_acl = "{}"
query_blacklist = "{}"
```

## Notes

- `query_*` must be valid JSON (for example `{}` or `{"enabled": true}`).
- Add indexes for fields like `username`, `clientid`, `resource_name`.
- Keep `blacklist_type` enum values consistent to avoid skipped records.
