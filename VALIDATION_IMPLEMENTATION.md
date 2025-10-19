# Parameter Validation Implementation

## Overview
Added parameter validation for the `/api/mqtt/blacklist/create` endpoint using the `validator` crate.

## Changes Made

### 1. Dependencies
- Added `validator = { version = "0.18", features = ["derive"] }` to `src/admin-server/Cargo.toml`

### 2. Custom Extractor
Created `src/admin-server/src/extractor.rs`:
- `ValidatedJson<T>` - A custom Axum extractor that automatically validates request parameters
- Integrates with `validator` crate's `Validate` trait
- Returns user-friendly error messages

### 3. Request Parameter Validation
Modified `src/admin-server/src/request/mqtt.rs`:
- Added `Validate` derive to `CreateBlackListReq`
- Validation rules:
  - **blacklist_type**: Length 1-50, must be "ClientId", "IpAddress", or "Username"
  - **resource_name**: Length 1-256
  - **end_time**: Must be > 0
  - **desc**: Max length 500

### 4. Handler Update
Modified `src/admin-server/src/mqtt/blacklist.rs`:
- Changed from `Json<CreateBlackListReq>` to `ValidatedJson<CreateBlackListReq>`
- Parameters are automatically validated before handler execution

## Validation Rules

| Field | Rules | Error Message |
|-------|-------|---------------|
| `blacklist_type` | Length: 1-50<br>Custom: must be ClientId/IpAddress/Username | "Blacklist type length must be between 1-50"<br>"Blacklist type must be ClientId, IpAddress or Username" |
| `resource_name` | Length: 1-256 | "Resource name length must be between 1-256" |
| `end_time` | Range: min=1 | "End time must be greater than 0" |
| `desc` | Max length: 500 | "Description length cannot exceed 500 characters" |

## Error Response Format

When validation fails, the API returns:
```json
{
  "code": 1,
  "message": "Parameter validation failed: <detailed_error_message>"
}
```

Example error messages:
- "Blacklist type must be ClientId, IpAddress or Username"
- "Resource name length must be between 1-256"
- "Description length cannot exceed 500 characters"

## Testing

### Run the Test Script
```bash
./test_blacklist_validation.sh
```

### Manual Testing Examples

**Valid Request:**
```bash
curl -X POST http://localhost:8030/api/mqtt/blacklist/create \
  -H "Content-Type: application/json" \
  -d '{
    "blacklist_type": "ClientId",
    "resource_name": "test_client",
    "end_time": 9999999999,
    "desc": "Test blacklist"
  }'
```

**Invalid blacklist_type:**
```bash
curl -X POST http://localhost:8030/api/mqtt/blacklist/create \
  -H "Content-Type: application/json" \
  -d '{
    "blacklist_type": "InvalidType",
    "resource_name": "test_client",
    "end_time": 9999999999,
    "desc": "Test"
  }'
```
Response:
```json
{
  "code": 1,
  "message": "Parameter validation failed: Blacklist type must be ClientId, IpAddress or Username"
}
```

**Empty resource_name:**
```bash
curl -X POST http://localhost:8030/api/mqtt/blacklist/create \
  -H "Content-Type: application/json" \
  -d '{
    "blacklist_type": "ClientId",
    "resource_name": "",
    "end_time": 9999999999,
    "desc": "Test"
  }'
```
Response:
```json
{
  "code": 1,
  "message": "Parameter validation failed: Resource name length must be between 1-256"
}
```

## Benefits

1. **Automatic Validation**: Parameters are validated before reaching handler logic
2. **Clear Error Messages**: Users get helpful feedback about what's wrong
3. **Reusable**: `ValidatedJson` extractor can be used for other endpoints
4. **Type Safety**: Validation rules are defined at compile time
5. **Reduced Boilerplate**: No need for manual validation code in each handler

## Next Steps

To add validation to other endpoints:

1. Add `#[derive(Validate)]` to the request struct
2. Add validation annotations to fields
3. Replace `Json<T>` with `ValidatedJson<T>` in the handler
4. That's it! Validation happens automatically

Example:
```rust
#[derive(Serialize, Deserialize, Validate)]
pub struct CreateUserReq {
    #[validate(length(min = 3, max = 20, message = "Username length must be between 3-20"))]
    pub username: String,
    
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

pub async fn user_create(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<CreateUserReq>,
) -> String {
    // Parameters are already validated
    // ... business logic
}
```

