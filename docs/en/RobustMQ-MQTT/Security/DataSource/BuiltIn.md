# Built-in Data Source (Meta Service)

The built-in data source stores authentication, authorization, and blacklist data directly in RobustMQ Meta Service.

## How to Use

- No extra configuration is required.
- Manage users, ACL, and blacklist through management APIs.
- Cache updates are real-time with no sync delay.
- CONNECT auth checks in-memory cache first.

## Notes

- Simple operations and fast onboarding.
- Suitable for quick rollout and small-to-medium deployments.
- If you already have a centralized identity system, you can migrate to external data sources later.
