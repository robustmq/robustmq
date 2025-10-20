# RobustMQ Docker Images

## 📦 依赖基础镜像

**用途：** 预编译所有 Rust 依赖，加速 CI/CD 构建

**镜像：** `ghcr.io/socutes/robustmq/rust-deps:latest`

**效果：** CI 构建时间从 15-18 分钟降到 2-3 分钟（5-10倍提升）

---

## 🚀 快速开始

### 1. 构建并推送镜像

```bash
# 登录 GitHub Container Registry
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin

# 构建并推送镜像
cd docker/
./build-and-push.sh

# 或使用自定义标签
./build-and-push.sh rust-1.90
./build-and-push.sh 2025-10-20
```

### 2. 在 GitHub Actions 中使用

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
      - run: cargo build --workspace
      - run: cargo nextest run --workspace
```

---

## 📋 文件说明

| 文件 | 用途 |
|------|------|
| `Dockerfile` | 生产环境镜像 |
| `Dockerfile.deps` | 依赖基础镜像（手动维护） |
| `build-and-push.sh` | 构建和推送脚本 |
| `README.md` | 本文件 |

---

## 🔄 何时更新镜像？

**需要更新：**
- 每 2-4 周（常规维护）
- Rust 版本升级
- 20+ 依赖变更
- CI 构建时间超过 8 分钟

**不需要更新：**
- 仅修改项目代码
- 少量依赖更新（Cargo 会增量处理）

---

## 🏷️ 版本标签

```bash
# 推荐标签
ghcr.io/socutes/robustmq/rust-deps:latest          # 最新版
ghcr.io/socutes/robustmq/rust-deps:rust-1.90       # Rust 版本
ghcr.io/socutes/robustmq/rust-deps:2025-10-20      # 日期版本
ghcr.io/socutes/robustmq/rust-deps:v0.2.0          # 发布版本
```

---

## 🔧 故障排查

### 构建失败
```bash
# 检查磁盘空间（需要 20GB+）
df -h

# 检查 Docker 是否运行
docker info

# 检查登录状态
docker images ghcr.io/socutes/robustmq/rust-deps
```

### 推送失败
```bash
# 重新登录
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin

# 检查 token 权限（需要 write:packages）
```

### CI 无法拉取镜像
```yaml
# 确保配置了 credentials
container:
  image: ghcr.io/socutes/robustmq/rust-deps:latest
  credentials:
    username: ${{ github.actor }}
    password: ${{ secrets.GITHUB_TOKEN }}
```

---

## 📊 性能对比

| 场景 | 无缓存 | 有缓存 | 提升 |
|------|--------|--------|------|
| 依赖下载 | 2-3 分钟 | 0 秒 | ✅ |
| 依赖编译 | 10-12 分钟 | 0 秒 | ✅ |
| 项目编译 | 2-3 分钟 | 2-3 分钟 | - |
| **总计** | **15-18 分钟** | **2-3 分钟** | **5-10x** |

---

## 💡 工作原理

1. **镜像包含：** Rust 1.90.0 + 所有依赖预编译
2. **CI 使用：** 直接拉取镜像，跳过依赖编译
3. **增量更新：** 即使镜像过期，Cargo 只重编译变化的部分
4. **智能缓存：** 90%+ 依赖命中缓存，仍比无缓存快 3-4 倍

---

**问题？** 查看 GitHub Actions 日志或提交 issue。