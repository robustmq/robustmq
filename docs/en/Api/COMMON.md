# RobustMQ Admin Server HTTP API Common Guide

## Overview

RobustMQ Admin Server is an HTTP management interface service, providing comprehensive management capabilities for RobustMQ clusters.

- **Base URL**: `http://localhost:8080`
- **API Prefix**: `/api` (all management interfaces use this prefix)
- **Request Method**: Mainly uses `POST` method
- **Data Format**: JSON
- **Response Format**: JSON

## API Documentation Navigation

- 📋 **[Cluster Management API](CLUSTER.md)** - Cluster configuration and status management
- 🔧 **[MQTT Broker API](MQTT.md)** - All MQTT broker related management interfaces

---

## Common Response Format

### Success Response
```json
{
  "code": 0,
  "message": "success",
  "data": {...}
}
```

### Error Response
```json
{
  "code": 500,
  "message": "error message",
  "data": null
}
```

### Paginated Response Format
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [...],
    "total_count": 100
  }
}
```

---

## Common Request Parameters

Most list query interfaces support the following common parameters:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `limit` | `u32` | No | Page size, default 10000 |
| `page` | `u32` | No | Page number, starts from 1, default 1 |
| `sort_field` | `string` | No | Sort field |
| `sort_by` | `string` | No | Sort order: asc/desc |
| `filter_field` | `string` | No | Filter field |
| `filter_values` | `array` | No | Filter value list |
| `exact_match` | `string` | No | Exact match: true/false |

### Pagination Parameters Example
```json
{
  "limit": 20,
  "page": 1,
  "sort_field": "create_time",
  "sort_by": "desc",
  "filter_field": "status",
  "filter_values": ["active"],
  "exact_match": "false"
}
```

---

## Basic Interface

### Service Status Query
- **Endpoint**: `GET /api/status`
- **Description**: Get service status and version information
- **Request Parameters**: None
- **Response Example**:
```json
"RobustMQ API v0.1.34"
```

### Service Version Query
- **Endpoint**: `GET /`
- **Description**: Get service version information (backward compatibility interface)
- **Request Parameters**: None
- **Response Example**:
```json
"RobustMQ API v0.1.34"
```

---

## Error Code Description

| Error Code | Description |
|------------|-------------|
| 0 | Request successful |
| 400 | Request parameter error |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Resource not found |
| 500 | Internal server error |

---

## Usage Examples

### Basic Request Example
```bash
# Get service version
curl -X GET http://localhost:8080/

# List query with pagination
curl -X POST http://localhost:8080/api/mqtt/user/list \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 10,
    "page": 1,
    "sort_field": "username",
    "sort_by": "asc"
  }'
```

### Error Handling Example
```bash
# When request fails, detailed error information is returned
{
  "code": 400,
  "message": "Invalid parameter: username is required",
  "data": null
}
```

---

## Notes

1. **Request Method**: Except for the root path `/` which uses GET method, all other interfaces use POST method
2. **Request Body**: Even for query operations, a JSON format request body needs to be sent
3. **Time Format**: 
   - Input time uses Unix timestamp (seconds)
   - Output time uses local time format string "YYYY-MM-DD HH:MM:SS"
4. **Pagination**: Page number `page` starts counting from 1
5. **Configuration Validation**: Resource creation validates configuration format correctness
6. **Access Control**: It's recommended to add appropriate authentication and authorization mechanisms in production environments
7. **Error Handling**: All errors return detailed error information for easy debugging
8. **Content Type**: Requests must set the `Content-Type: application/json` header

---

## Development and Debugging

### Start Service
```bash
# Start admin-server
cargo run --bin admin-server

# Or use compiled binary
./target/release/admin-server
```

### Test Connection
```bash
# Test if service is running normally
curl -X GET http://localhost:8080/
```

### Log Viewing
The service outputs detailed log information during runtime, including:
- Request paths and parameters
- Response status and data
- Error messages and stack traces

---

*Documentation Version: v4.0*  
*Last Updated: 2025-09-20*  
*Based on Code Version: RobustMQ Admin Server v0.1.34*
