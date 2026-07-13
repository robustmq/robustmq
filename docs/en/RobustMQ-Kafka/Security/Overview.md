# Security Overview

RobustMQ Kafka implements Kafka's security-related APIs at the protocol layer and persists all security data — authentication credentials, ACLs, quotas, delegation tokens — in the Raft-based meta-service. This page outlines the current security boundary and links to the topic pages.

> To be honest up front: **only SASL/SCRAM authentication is enforced end to end.** ACLs, quotas, and delegation tokens all ship with full, protocol-compatible management interfaces (create/describe/delete, persisted and broadcast across the cluster), but they are **not yet wired into the request path for enforcement**. They are "manageable metadata" today, not mechanisms that actually block, throttle, or authorize.

## Capability matrix

| Capability | APIs | Status | Notes |
|---|---|---|---|
| SASL/SCRAM authentication | SaslHandshake / SaslAuthenticate / AlterUserScramCredentials / DescribeUserScramCredentials | ✅ Enforced | SCRAM-SHA-256 / SCRAM-SHA-512 only; when enabled, unauthenticated connections may send handshake requests only |
| ACL authorization | CreateAcls / DescribeAcls / DeleteAcls | 🟡 Manageable, not enforced | create/describe/delete and persist, but the request path never reads ACLs — nothing is blocked |
| Client quotas | AlterClientQuotas / DescribeClientQuotas | 🟡 Manageable, not enforced | `client-id` entity only; setting a quota does not throttle |
| Delegation tokens | Create / Renew / Expire / DescribeDelegationToken | 🟡 Metadata management | the HMAC is not used for authentication; owner is fixed to `User:ANONYMOUS` |

## Persistence path

All security data follows one path:

1. A broker receives a management request (e.g. `AlterUserScramCredentials`), validates it, and forwards it to meta-service over gRPC;
2. meta-service writes it into RocksDB through **Raft** (strong consistency);
3. on success it **broadcasts a cache-update** notification to every broker;
4. each broker **loads the full set once at startup** from meta-service and applies broadcast deltas at runtime.

So credentials, ACLs, and quotas are strongly consistent and identical across all nodes.

## Default state

SASL is **off by default** (`kafka.sasl.enabled = false`). When off, all connections are treated as authenticated with no handshake. When on, the default enabled mechanism list is `["SCRAM-SHA-256", "SCRAM-SHA-512"]`.

## Topic pages

- [SASL/SCRAM Authentication](./Authentication-SASL-SCRAM.md)
- [ACL Authorization](./Authorization-ACL.md)
- [Client Quotas](./Quota.md)
- [Delegation Tokens](./DelegationToken.md)

> Storage and coordination mechanics are in [System Architecture](../SystemArchitecture.md) and [Storage Engine](../Storage.md).
