# Authentication

Authentication can effectively prevent illegal client connections. RobustMQ provides a flexible and secure authentication mechanism that supports multiple authentication methods and storage backends.

## Overview

RobustMQ MQTT Broker's authentication system has the following features:

- **Multiple Authentication Methods**: Supports plaintext authentication, JWT authentication, HTTP authentication, etc.
- **Flexible Storage Backends**: Supports Placement Center, MySQL and other storage methods
- **Super User Mechanism**: Supports super user privileges that can bypass ACL checks
- **Password-free Login**: Supports password-free login configuration in test environments
- **User Caching**: Provides user information caching mechanism to improve authentication performance

Some authentication methods and storage backend support are under development.

## Authentication Flow

RobustMQ's authentication flow proceeds in the following order:

1. **Password-free Login Check**: If `secret_free_login` is enabled, skip authentication and allow connection directly
2. **Username and Password Verification**: Verify the username and password provided by the client
3. **User Information Query**: Get user information from cache or storage layer
4. **Authentication Result Return**: Return authentication success or failure

## Configure Authentication

### Basic Configuration

Set authentication-related parameters in the `server.toml` configuration file:

```toml
[mqtt_security]
# Whether to enable password-free login
secret_free_login = false

[mqtt_auth_storage]
# Authentication storage type: placement | mysql
storage_type = "placement"
# MySQL connection address (when using MySQL storage)
mysql_addr = "mysql://username:password@localhost:3306/mqtt"
```

### Storage Backend Configuration

#### Placement Center Storage

Use RobustMQ's built-in Placement Center as the storage backend:

```toml
[mqtt_auth_storage]
storage_type = "placement"
```

#### MySQL Storage

Use MySQL database as the storage backend:

```toml
[mqtt_auth_storage]
storage_type = "mysql"
mysql_addr = "mysql://root:password@localhost:3306/mqtt"
```

Create the corresponding data table:

```sql
CREATE TABLE `mqtt_user` (
    `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
    `username` varchar(100) DEFAULT NULL,
    `password` varchar(100) DEFAULT NULL,
    `salt` varchar(35) DEFAULT NULL,
    `is_superuser` tinyint(1) DEFAULT 0,
    `created` datetime DEFAULT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `mqtt_username` (`username`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

## User Management

### Create User

#### Using robust-ctl Command Line Tool

```bash
# Create regular user
robust-ctl mqtt user create --username testuser --password testpass

# Create super user
robust-ctl mqtt user create --username admin --password adminpass --is-superuser
```

#### Using HTTP API

```bash
# Create regular user
curl -X POST http://localhost:8080/api/mqtt/user/create \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "testpass",
    "is_superuser": false
  }'

# Create super user
curl -X POST http://localhost:8080/api/mqtt/user/create \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "adminpass",
    "is_superuser": true
  }'

# Query user list
curl -X POST http://localhost:8080/api/mqtt/user/list \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 10,
    "page": 1
  }'

# Delete user
curl -X POST http://localhost:8080/api/mqtt/user/delete \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser"
  }'
```

#### Direct MySQL Operations

```sql
-- Create regular user
INSERT INTO mqtt_user (username, password, is_superuser)
VALUES ('testuser', 'testpass', 0);

-- Create super user
INSERT INTO mqtt_user (username, password, is_superuser)
VALUES ('admin', 'adminpass', 1);
```

### Query Users

```bash
# Query all users
robust-ctl mqtt user list
```

### Delete User

```bash
# Delete user
robust-ctl mqtt user delete --username testuser
```

## Authentication Methods

### Plaintext Authentication

The most commonly used authentication method, directly verifying username and password:

```rust
// Client provides username and password when connecting
let login = Login {
    username: "testuser".to_string(),
    password: "testpass".to_string(),
};
```

### Password-free Login

Suitable for test environments, skips username and password verification:

```toml
[mqtt_security]
secret_free_login = true
```

**Note: Please make sure to disable password-free login in production environments.**

## Super User

Super users have special privileges and can bypass ACL checks, having full access to all topics.

### Super User Features

- Bypass all ACL permission checks
- Can publish and subscribe to any topic
- Not subject to blacklist restrictions (unless explicitly added to the blacklist)
- Suitable for administrator accounts

### Set Super User

Set `is_superuser` to `true` when creating a user:

```sql
INSERT INTO mqtt_user (username, password, is_superuser)
VALUES ('admin', 'admin123', 1);
```

## Common Questions

### Q: What to do if authentication fails?

A: Check the following aspects:

1. **Are the username and password correct**
2. **Does the user exist**: Use `robust-ctl mqtt user get` to query
3. **Is the configuration correct**: Check the authentication configuration in `server.toml`
4. **Is the storage layer normal**: Check if MySQL or Placement Center is running normally

### Q: How to reset user password?

A: You can reset through the following methods:

```bash
# Using command line tool
robust-ctl mqtt user update --username testuser --password newpass

# Or directly modify the database
UPDATE mqtt_user SET password = 'newpass' WHERE username = 'testuser';
```

### Q: Are super users subject to blacklist restrictions?

A: Super users are bypassed in ACL checks, but are still subject to blacklist check restrictions.

### Q: How to enable debug logs?

A: Set the log level in the configuration file:

```toml
[log]
level = "debug"
```

### Q: How to update authentication cache?

A: RobustMQ will periodically synchronize storage layer data to cache, and you can also manually refresh through API:

```bash
curl -X POST http://localhost:8080/api/mqtt/cache/refresh
```

## Troubleshooting

### Connection Rejected

1. Check if username and password are correct
2. Confirm that the user exists and is not disabled
3. Check if added to blacklist
4. View RobustMQ logs for detailed error information

### Authentication Performance Issues

1. Monitor database connection pool status
2. Check cache hit rate
3. Optimize database query performance
4. Consider increasing cache size

### Configuration Not Taking Effect

1. Confirm the configuration file path is correct
2. Restart RobustMQ service
3. Check if configuration file syntax is correct
4. View startup logs to confirm configuration loading status
