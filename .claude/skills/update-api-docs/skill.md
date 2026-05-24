---
name: update-api-docs
description: Audit and update the HTTP API documentation under docs/zh/Api/ and docs/en/Api/ against the actual route definitions in src/admin-server/src/. Uses path.rs as the single source of truth. Fixes wrong URI prefixes, non-existent routes, wrong request/response fields, and syncs the English docs to match the Chinese ones.
---

# update-api-docs

Perform a full audit of the admin-server HTTP API documentation against the actual source code, then fix all discrepancies and sync both Chinese and English versions.

## Usage

```
/update-api-docs
/update-api-docs <doc_file>
```

**Examples**

```
/update-api-docs
/update-api-docs docs/zh/Api/MQTT.md
/update-api-docs docs/zh/Api/CLUSTER.md
```

When no file is specified, audit all doc files under `docs/zh/Api/` and `docs/en/Api/`.

---

## Source of Truth

- **Route paths**: `src/admin-server/src/path.rs` — all path constants are defined here
- **Handler structs**: `src/admin-server/src/` subdirectories — request/response types per feature area
- **Route nesting**: `src/admin-server/src/server.rs` — all routes under `GET/POST /api/...` are nested via `.nest("/api", self.api_route())`; health and debug routes are mounted outside `/api`

**Key path facts to remember:**
- Health routes have **no** `/api` prefix: `/health/ready`, `/health/node`, `/health/cluster`
- Debug routes have **no** `/api` prefix: `/debug/pprof/flamegraph`, `/metrics`
- Cluster index is `GET /` (no `/api` prefix)
- Tenant routes: `/api/cluster/tenant/*` (NOT `/api/tenant/*`)
- User routes: `/api/cluster/user/*` (NOT `/api/mqtt/user/*`)
- Message routes: `/api/cluster/message/*` (NOT `/api/mqtt/message/*`)
- Connector routes: `/api/cluster/connector/*` (NOT `/api/mqtt/connector/*`)
- MQ9 routes: `/api/mq9/mail/list`, `/api/mq9/agent/list`

---

## Execution Flow

### Step 1 — Read path.rs

Read `src/admin-server/src/path.rs` in full. This file contains all route path constants. Extract every path constant into a checklist — these are the **only** valid routes.

### Step 2 — Read server.rs

Read `src/admin-server/src/server.rs` to understand which paths get the `/api` prefix and which do not. Note which routers are nested under `.nest("/api", ...)`.

### Step 3 — Read handler files

For each handler subdirectory (`cluster/`, `mqtt/`, `nats/`, `mcp/`, etc.), read the relevant handler files to extract:
- Exact request struct field names and types
- Exact response struct field names and types
- Any validation constraints (e.g. `length(1..=256)`)

Focus on the structs decorated with `#[derive(Deserialize)]` (request) and `#[derive(Serialize)]` (response).

### Step 4 — Read doc files

For each doc file to audit:
1. Extract every documented endpoint: method + path
2. Extract every documented request field
3. Extract every documented response field

### Step 5 — Compare and identify discrepancies

Check each documented endpoint against the source of truth:

| Issue type | How to detect |
|------------|--------------|
| Wrong URI prefix | Path in doc doesn't match any constant in path.rs |
| Non-existent route | Path in doc has no corresponding constant in path.rs |
| Extra request field | Field in doc not present in the handler's request struct |
| Missing request field | Required field in handler struct missing from doc |
| Wrong response field | Field in doc not in response struct |
| Missing response field | Key response field not documented |
| Wrong HTTP method | GET vs POST mismatch |
| Wrong field type | e.g. doc says `string` but struct says `u64` |

### Step 6 — Fix all discrepancies

Fix all identified issues in the Chinese doc files first:

- **Wrong URI prefix**: Replace with the correct path from path.rs
- **Non-existent route**: Replace with the actual route, or add a clear note if the route was removed
- **Extra request field** (e.g. `retain` in SendMessageReq): Remove from doc
- **Missing required field**: Add to the parameter table and example JSON
- **Wrong field type**: Correct to match the struct definition
- **Curl examples**: Update all example URLs to match fixed paths, update example bodies to match fixed field lists

After fixing the Chinese docs, sync the English docs to match:
- All endpoint paths must be identical to the Chinese version
- All field names and types must be identical
- Descriptions should be the English equivalent

### Step 7 — Verify

After editing, grep the fixed doc files to confirm:
- No old wrong paths remain
- No removed fields remain in request examples
- All curl examples use the correct paths

---

## What NOT to change

- Do not alter correct endpoints
- Do not change prose descriptions unless they are factually wrong
- Do not reformat sections that don't contain errors
- Do not change connector type reference tables or enum value tables (these are application-level, not route-level)
- Do not modify zh/en sidebar files unless a new doc file was created

---

## New doc files

If a route area exists in path.rs but has no corresponding doc file (e.g. a new protocol like mq9):
1. Create `docs/zh/Api/<Name>.md` with all routes documented
2. Create `docs/en/Api/<Name>.md` as the English version
3. Add both to the appropriate sidebars in `docs/.vitepress/src/sidebars/zh.mts` and `en.mts`

---

## Output Format

After completing the audit, report:

```
## API Doc Audit Results

### Files checked
- docs/zh/Api/CLUSTER.md
- docs/zh/Api/MQTT.md
- docs/zh/Api/Connector.md
- docs/en/Api/CLUSTER.md
- docs/en/Api/MQTT.md
- docs/en/Api/Connector.md

### Discrepancies found and fixed

| File | Section | Issue | Fix |
|------|---------|-------|-----|
| zh/MQTT.md | §6.1 | Wrong path: /api/mqtt/user/list | → /api/cluster/user/list |
| zh/MQTT.md | §11.1 | Extra field: retain | Removed |
| zh/CLUSTER.md | §8 | Non-existent: GET /cluster/healthy | → GET /health/node |
| ... | | | |

### New files created
- docs/zh/Api/MQ9.md
- docs/en/Api/MQ9.md

### No issues found in
- docs/zh/Api/COMMON.md
```
