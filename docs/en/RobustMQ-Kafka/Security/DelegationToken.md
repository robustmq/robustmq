# Delegation Tokens

RobustMQ Kafka implements the metadata-management APIs for delegation tokens (KIP-48): `CreateDelegationToken` / `RenewDelegationToken` / `ExpireDelegationToken` / `DescribeDelegationToken`.

> **Current state (please note): this is pure metadata management. The token HMAC is not used for authentication** — neither the broker nor any other path verifies a token's HMAC during authentication. SASL currently supports SCRAM only; there is no delegation-token-based authentication mechanism. These APIs provide protocol-compatible token lifecycle management, but the tokens themselves cannot currently be used to log in.

## Behavior at a glance

| Aspect | Detail |
|---|---|
| Signing key | the broker **self-generates** a 32-byte random key, stored in meta-service resource config and shared across nodes |
| HMAC | computed as `HmacSha256(secret, token_id)`, used only as the lookup key for Renew/Expire and echoed in responses |
| Ownership | the requester is fixed to the placeholder principal `User:ANONYMOUS` (no principal authentication); without an explicit owner, ownership falls back to the requester |
| Verification | Renew/Expire locate a token purely by the presented HMAC and do **not** verify the caller is the owner/renewer |
| Expiry reaping | a background reaper runs every 60 seconds, deleting tokens past `max_timestamp_ms` |

## CLI examples

In secure mode (SASL enabled), `kafka-delegation-tokens.sh` needs `--command-config` to supply the SASL configuration:

```bash
# Create a token
kafka-delegation-tokens.sh --bootstrap-server localhost:9092 \
  --command-config client-sasl.properties \
  --create --max-life-time-period -1

# Describe
kafka-delegation-tokens.sh --bootstrap-server localhost:9092 \
  --command-config client-sasl.properties --describe

# Renew / expire
kafka-delegation-tokens.sh --bootstrap-server localhost:9092 \
  --command-config client-sasl.properties \
  --renew --renew-time-period -1 --hmac <HMAC>
```

Token metadata is likewise persisted through Raft and broadcast to every broker's cache (see [Overview](./Overview.md#persistence-path)).

## Limitations at a glance

| Limitation | Detail |
|---|---|
| Not used for auth | the token HMAC does not log in; there is no SASL/SCRAM delegation-token re-auth flow |
| No ownership authz | ownership is fixed to `User:ANONYMOUS`; renew/expire do not verify the caller |
| Metadata only | provides only token create/renew/expire/describe and automatic reaping |
