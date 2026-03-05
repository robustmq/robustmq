# Data Source

RobustMQ MQTT supports multiple data sources for authentication, authorization, and blacklist data.

## What Is a Data Source

A data source is where identity and policy data comes from:

- users
- ACL rules
- blacklist entries

Broker loads these data sets and uses them in the connection auth and access-control path.

## Why Multiple Data Sources

- Different teams already store identity data in different systems.
- Some scenarios prefer built-in metadata, others require external system integration.
- It enables gradual migration: start built-in, move to external sources when needed.
- It avoids hard-binding to a single storage model.

## How to Use

Typical steps:

1. Configure `storage_type` through the management console or CLI.
2. Fill the matching config block (for example `mysql_config`, `redis_config`).
3. For SQL-based sources, configure queries and ensure result mappings match expected contracts.
4. Validate auth result and cache sync behavior after startup.

## Runtime Principle

The auth path is cache-first, with different update strategies per source:

1. CONNECT auth checks in-memory cache first.
2. Built-in data source (Meta Service) updates cache in real time (no delay).
3. External data sources (MySQL/PostgreSQL/Redis/HTTP) update cache by periodic sync.
4. External sources are used for sync/management, not queried on every CONNECT.

By moving the hot path to memory, broker remains stable under connection bursts.  
By default, external source sync has around a 5-second delay; built-in source does not.

## Currently Supported Data Sources

- Built-in Data Source (Meta Service)
- MySQL
- PostgreSQL
- Redis
- HTTP

## Design Goals

- No forced fixed table schema.
- SQL/query-driven adaptation for existing systems.
- Cache-first auth hot path with manageable consistency trade-offs.

## Subpages

- [Built-in Data Source (Meta Service)](./DataSource/BuiltIn.md)
- [MySQL Data Source](./DataSource/MySQL.md)
- [PostgreSQL Data Source](./DataSource/PostgreSQL.md)
- [Redis Data Source](./DataSource/Redis.md)
- [HTTP Data Source](./DataSource/HTTP.md)
