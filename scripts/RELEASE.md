# Release Script 使用说明

## 快速开始

### 1. 发布新版本

```bash
# 使用 Cargo.toml 中的版本发布当前平台
./scripts/release.sh

# 指定版本发布
./scripts/release.sh --version v0.1.30
```

### 2. 只上传本系统安装包

```bash
# 使用 Cargo.toml 版本上传（版本必须已存在）
./scripts/release.sh --upload-only

# 指定版本上传到已存在的版本
./scripts/release.sh --upload-only --version v0.1.30
```

## 前置条件

1. **设置 GitHub Token**
```bash
export GITHUB_TOKEN="your_github_token_here"
```

2. **安装依赖**
- `curl` - API 请求
- `jq` - JSON 解析

## 完整用法

```bash
./scripts/release.sh [OPTIONS]

选项:
  -h, --help              显示帮助
  -v, --version VERSION   指定版本（默认从 Cargo.toml 读取）
  -t, --token TOKEN      GitHub Token
  --upload-only          仅上传到现有版本
```

## 使用场景

### 场景1: 创建新版本发布
```bash
# 使用 Cargo.toml 版本创建发布
./scripts/release.sh

# 指定版本创建发布
./scripts/release.sh --version v0.1.31
```

### 场景2: 为现有版本补充平台包
```bash
# 使用 Cargo.toml 版本上传（版本必须已存在）
./scripts/release.sh --upload-only

# 在 macOS 上为指定版本添加 macOS 包
./scripts/release.sh --upload-only --version v0.1.31

# 在 Linux 上为指定版本添加 Linux 包  
./scripts/release.sh --upload-only --version v0.1.31
```

### 场景3: 环境变量方式
```bash
export GITHUB_TOKEN="ghp_xxxx"
export VERSION="v0.1.31"  # 可选，不设置则使用 Cargo.toml 版本
./scripts/release.sh
```

## 注意事项

- ✅ 不指定 `--version` 时自动使用 Cargo.toml 中的版本
- ✅ 总是构建当前系统平台
- ✅ 总是包含前端构建
- ❌ `--upload-only` 模式下版本必须已存在
- ❌ 不支持多平台一次性构建

## 错误处理

**版本不存在**
```bash
❌ Release v0.1.99 does not exist
# 解决：先创建版本或去掉 --upload-only
```

**Token 缺失**
```bash
❌ GitHub token is required
# 解决：export GITHUB_TOKEN="your_token"
```
