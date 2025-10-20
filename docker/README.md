# RobustMQ Docker Images

## 1. Build Dependency Image (CI Acceleration)

```bash
# Login
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin

# Build dependency image
cd docker/
./build-and-push.sh
```

**Purpose:** Pre-compile all Rust dependencies to accelerate CI/CD builds

**Image:** `ghcr.io/socutes/robustmq/rust-deps:latest`

## 2. Build Application Image (Production Deployment)

```bash
# Build RobustMQ application image
docker build -f docker/Dockerfile -t robustmq:latest .

# Run container
docker run -d --name robustmq robustmq:latest
```

**Purpose:** Deploy RobustMQ service in production environment

**Image:** `robustmq:latest`

## Use Dependency Image in CI

```yaml
container:
  image: ghcr.io/socutes/robustmq/rust-deps:latest
  credentials:
    username: ${{ github.actor }}
    password: ${{ secrets.GITHUB_TOKEN }}
```

## When to Update Dependency Image

- Every 2-4 weeks
- Rust version upgrades
- 20+ dependency changes
- CI takes longer than 8 minutes

## Troubleshooting

```bash
# Check disk space
df -h

# Check Docker
docker info

# Re-login
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin
```