# RobustMQ Docker Images

This directory contains Docker-related files for RobustMQ project.

## 📦 Available Images

### 1. Dependency Base Image (`rust-deps`)

**Purpose:** Pre-compiled Rust dependencies for CI/CD acceleration

**Image:** `ghcr.io/socutes/robustmq/rust-deps:latest`

**What's Inside:**
- Rust 1.90.0 toolchain
- All system dependencies (protobuf, llvm, clang, lld, etc.)
- All Cargo dependencies pre-compiled (~300 crates)
- Build tools (cargo-nextest, sccache)

**Build Time:** ~20-40 minutes (first time)

**Image Size:** ~8-10 GB

---

## 🚀 Quick Start

### Build Dependency Image Locally

```bash
# 1. Login to GitHub Container Registry
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin

# 2. Build and push the image
cd docker/
./build-and-push.sh

# 3. (Optional) Build with custom tag
./build-and-push.sh rust-1.90
./build-and-push.sh 2025-10-20
```

### Use in GitHub Actions

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    container:
      image: ghcr.io/socutes/robustmq/rust-deps:latest
      credentials:
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Build project
        run: cargo build --workspace
        # ⚡ Dependencies already compiled!
      
      - name: Run tests
        run: cargo nextest run --workspace
```

---

## 📋 Files in This Directory

| File | Purpose |
|------|---------|
| `Dockerfile` | Production image for RobustMQ application |
| `Dockerfile.deps` | Dependency base image (maintained manually) |
| `build-and-push.sh` | Script to build and push `rust-deps` image |
| `README.md` | This file |

---

## 🔄 Update Strategy

### When to Rebuild `rust-deps` Image?

✅ **Should rebuild when:**
- Cargo.lock has 20+ dependency changes
- Rust version upgrades (e.g., 1.90 → 1.91)
- System dependencies change (protobuf version, etc.)
- CI build time consistently exceeds 8 minutes

❌ **No need to rebuild when:**
- Only project code changes
- 1-5 dependency updates (Cargo handles incrementally)
- Documentation changes
- Configuration file changes

### How Often?

**Recommended frequency:**
- **Active development:** Every 2-4 weeks
- **Stable phase:** Monthly
- **On-demand:** When CI becomes slow

### Monitoring CI Performance

Add this to workflows to track dependency cache health:

```yaml
- name: Check cache health
  run: |
    START=$(date +%s)
    cargo build --workspace
    END=$(date +%s)
    DURATION=$((END - START))
    echo "Build time: ${DURATION}s"
    if [ $DURATION -gt 480 ]; then
      echo "⚠️ Build took >8min, consider updating rust-deps image"
    fi
```

---

## 🏷️ Version Tagging Strategy

### Recommended Tags

| Tag Pattern | Use Case | Example |
|------------|----------|---------|
| `latest` | Development branches | Always up-to-date |
| `rust-X.Y` | Rust version pin | `rust-1.90`, `rust-1.91` |
| `YYYY-MM-DD` | Date-based versions | `2025-10-20` |
| `vX.Y.Z` | Release versions | `v0.1.35`, `v0.2.0` |

### Example Workflow Usage

```yaml
# Development - use latest
container:
  image: ghcr.io/socutes/robustmq/rust-deps:latest

# Release - use pinned version
container:
  image: ghcr.io/socutes/robustmq/rust-deps:v0.2.0
```

---

## 💡 How It Works

### Architecture

```
┌─────────────────────────────────────────┐
│  rust-deps Image                        │
│  ├─ Rust 1.90.0                         │
│  ├─ System deps (protobuf, etc.)        │
│  ├─ 300+ dependencies pre-compiled ✅   │
│  └─ cargo-nextest, sccache              │
└─────────────────────────────────────────┘
              ↓ Pull in CI
┌─────────────────────────────────────────┐
│  GitHub Actions Container               │
│  ├─ Checkout code                       │
│  ├─ cargo build (only project code)     │
│  └─ cargo test                          │
│                                          │
│  Time: 2-3 minutes ⚡                   │
│  vs 15-18 minutes without cache 🐌      │
└─────────────────────────────────────────┘
```

### What Happens When Dependencies Update?

**Scenario:** You updated 10 dependencies in Cargo.lock

```bash
# In CI container (using rust-deps:latest)
cargo build --workspace

# Cargo intelligently handles this:
✅ 290 dependencies → Use cached (0s)
⚠️  10 dependencies  → Download + compile (~1-2 min)
🔨 Project code     → Compile (~2 min)

# Total: ~3-4 minutes (still 4x faster than no cache!)
```

**Key Point:** Even with outdated image, you still benefit from 90%+ cache hit rate!

---

## 🔧 Troubleshooting

### Image Too Large

```bash
# Check image size
docker images ghcr.io/socutes/robustmq/rust-deps:latest

# If >15GB, consider:
# 1. Clean up old layers
docker builder prune --all

# 2. Rebuild from scratch
docker build --no-cache -f docker/Dockerfile.deps -t IMAGE .
```

### Build Fails During `cargo chef cook`

```bash
# Common causes:
# 1. Cargo.toml syntax errors
# 2. Missing build.rs files
# 3. Workspace structure changes

# Debug:
docker build -f docker/Dockerfile.deps --target planner -t debug .
docker run --rm -it debug /bin/bash
# Then manually run: cargo chef prepare
```

### Push to GHCR Fails

```bash
# Ensure you're logged in
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin

# Verify token has 'write:packages' permission
# Create token at: https://github.com/settings/tokens

# Check package visibility
# Visit: https://github.com/users/YOUR_USERNAME/packages/container/robustmq%2Frust-deps/settings
```

### CI Cannot Pull Image

```yaml
# Make sure credentials are set
container:
  image: ghcr.io/socutes/robustmq/rust-deps:latest
  credentials:
    username: ${{ github.actor }}
    password: ${{ secrets.GITHUB_TOKEN }}  # ← Required!

# Check package visibility (should be public or accessible to repo)
```

---

## 📊 Performance Comparison

### Without Dependency Cache

```
Download dependencies:     2-3 minutes
Compile dependencies:     10-12 minutes
Compile project code:      2-3 minutes
─────────────────────────────────────
Total:                    15-18 minutes 🐌
```

### With Dependency Cache (Image Fresh)

```
Download dependencies:     0 seconds ✅
Compile dependencies:      0 seconds ✅
Compile project code:      2-3 minutes
─────────────────────────────────────
Total:                     2-3 minutes ⚡ (5-6x faster!)
```

### With Dependency Cache (Image 1 Month Old)

```
Download dependencies:     20-30 seconds (10 updated deps)
Compile dependencies:      1-2 minutes (10 updated deps)
Compile project code:      2-3 minutes
─────────────────────────────────────
Total:                     4-6 minutes ⚡ (3x faster!)
```

---

## 🎯 Best Practices

### For Maintainers

1. **Calendar Reminder:** Set monthly reminder to rebuild image
2. **Monitor CI Times:** Watch for consistent slowdowns
3. **Version Tags:** Use semantic versions for releases
4. **Document Updates:** Note major dependency changes in commit messages

### For Contributors

1. **Don't Worry:** Image doesn't need to be perfectly in sync
2. **Report Slowness:** If CI takes >10 minutes, notify maintainers
3. **Local Development:** Use normal `cargo build` (no special image needed)

### For Release Managers

1. **Pin Versions:** Use tagged images for release branches
2. **Test Image:** Verify new image works before pushing
3. **Backup Tags:** Keep previous versions for rollback

---

## 📚 Related Documentation

- [GitHub Actions Workflows](../.github/workflows/)
- [Build Scripts](../scripts/)
- [Project README](../README.md)

---

## 🤝 Contributing

If you improve the dependency image or build process:

1. Test locally first
2. Document changes in this README
3. Update version tags appropriately
4. Notify team in PR description

---

**Questions?** Open an issue or ask in discussions!
