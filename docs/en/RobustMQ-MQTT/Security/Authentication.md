# Authentication

Authentication verifies client identity before a connection is fully accepted by the broker.

## What Authentication Does

In MQTT, authentication is the first gate in the CONNECT path:

- It decides whether a client is allowed to establish a session.
- It prevents unauthorized connections from consuming broker resources.
- It provides identity context for downstream ACL and blacklist checks.

Authentication answers "who are you and can you enter", while authorization answers "what can you do after entering".

## Capabilities

- **Identity verification** for incoming clients.
- **Connection admission control** with explicit pass/fail outcomes.
- **Extensible authenticator model** to support more auth types over time.
- **Cache-first hot path** to keep high-frequency auth checks fast and stable.
- **Integration with ACL and blacklist** through shared runtime context.

## Login Authentication Flow (Chained Model)

Authentication follows a chained model where each authenticator is a node in the flow.

### Steps

1. Client sends `CONNECT` with login information.
2. Broker performs basic validation (protocol fields, client info, preconditions).
3. Broker selects authenticator(s) according to config order.
4. Authenticator executes verification (for example Password/JWT/HTTP-based verification).
5. If auth succeeds, broker continues with session setup; if auth fails, broker returns CONNECT failure.
6. Auth result is written into connection context for ACL, blacklist, metrics, and audit paths.

### Result Semantics

- **Allow**: continue connection setup.
- **Deny**: reject the connection with failure reason.
- **Error**: treated as auth failure to avoid accidental bypass.

## Current Implementation Status

The current production-ready primary method is Password authentication.

## Currently Supported Authentication Methods

- [Password Authentication](./Authentication-Password.md)

## Future Extensions

- [JWT Authentication](./Authentication-JWT.md) (placeholder, to be completed)
