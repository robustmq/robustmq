# RobustMQ Docker Quick Guide (GHCR only)

## Build Dependency Image (CI acceleration, GHCR)

```bash
# Login to GHCR (requires GITHUB_TOKEN)
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin

# Build and push dependency cache image (default: latest; or specify a tag)
cd docker/
./build-and-push.sh             # pushes ghcr.io/<org>/robustmq/rust-deps:latest
./build-and-push.sh 2025-10-20  # example: push with a specific tag
```

Example image: `ghcr.io/socutes/robustmq/rust-deps:<tag>`

## Build and Push Application Image (script, GHCR)

Login:
```bash
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin
```

Use the script:
```bash
./docker/build-and-push-app.sh --org socutes --registry ghcr --push-latest
# The script auto-detects version from Cargo.toml (workspace.package.version)
```