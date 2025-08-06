# RobustMQ Release Guide

This document describes how to use the `release.sh` script to automate RobustMQ releases to GitHub.

## 📋 Feature Overview

The `release.sh` script provides a complete automated release workflow:

1. **Version Extraction** - Automatically extract version information from `Cargo.toml`
2. **Release Creation** - Create corresponding release versions on GitHub
3. **Package Building** - Call `build.sh` to build multi-platform binary packages
4. **Asset Upload** - Upload tar.gz packages to GitHub release

## 🔧 Prerequisites

### Required Tools
- `curl` - For GitHub API calls
- `jq` - For JSON data processing
- `git` - Version control
- Rust toolchain (if building is required)

### GitHub Configuration
1. **Create Personal Access Token**
   - Visit: https://github.com/settings/tokens
   - Click "Generate new token (classic)"
   - Select permissions: `repo` (Full control of private repositories)
   - Copy the generated token

2. **Set Environment Variable**
   ```bash
   export GITHUB_TOKEN="your_github_token_here"
   ```

## 🚀 Usage

### Basic Usage

```bash
# 1. Set GitHub Token
export GITHUB_TOKEN="ghp_xxxxxxxxxxxxxxxxxxxx"

# 2. Release current version from Cargo.toml
./scripts/release.sh

# 3. Check build results
ls -la build/
```

### Advanced Usage

#### Release Specific Version
```bash
./scripts/release.sh --version v0.1.30
```

#### Build Specific Platform Only
```bash
./scripts/release.sh --platform linux-amd64
```

#### Dry Run Mode (Preview Operations)
```bash
./scripts/release.sh --dry-run --verbose
```

#### Force Recreate Existing Release
```bash
./scripts/release.sh --force
```

#### Skip Build, Upload Existing Packages Only
```bash
./scripts/release.sh --skip-build
```

## 📖 Command Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `-h, --help` | Show help message | - |
| `-v, --version VERSION` | Specify release version | Extract from Cargo.toml |
| `-p, --platform PLATFORM` | Target platform | all |
| `-t, --token TOKEN` | GitHub Personal Access Token | $GITHUB_TOKEN |
| `-r, --repo REPO` | GitHub repository | robustmq/robustmq |
| `--dry-run` | Dry run mode | false |
| `--force` | Force recreate release | false |
| `--skip-build` | Skip build step | false |
| `--verbose` | Verbose output | false |

## 🌍 Supported Platforms

| Platform ID | Description |
|-------------|-------------|
| `all` | Build for all supported platforms |
| `linux-amd64` | Linux x86_64 |
| `linux-arm64` | Linux ARM64 |
| `darwin-amd64` | macOS x86_64 |
| `darwin-arm64` | macOS ARM64 (Apple Silicon) |
| `windows-amd64` | Windows x86_64 |

## 🔄 Typical Release Workflow

### 1. Prepare for Release

```bash
# Ensure on main branch with latest code
git checkout main
git pull origin main

# Check current version
grep "version =" Cargo.toml

# Verify build works
cargo test
cargo build --release
```

### 2. Execute Release

```bash
# Set GitHub Token
export GITHUB_TOKEN="your_token"

# Perform dry run first
./scripts/release.sh --dry-run --verbose

# Execute actual release after confirmation
./scripts/release.sh --verbose
```

### 3. Verify Release

```bash
# Check GitHub Release page
# https://github.com/robustmq/robustmq/releases

# Verify package integrity
cd build/
sha256sum -c robustmq-v*.tar.gz.sha256
```

## 🗂️ Generated File Structure

After successful release, the following files will be generated in GitHub Release:

```
robustmq-v0.1.29-linux-amd64.tar.gz
robustmq-v0.1.29-linux-amd64.tar.gz.sha256
robustmq-v0.1.29-linux-arm64.tar.gz
robustmq-v0.1.29-linux-arm64.tar.gz.sha256
robustmq-v0.1.29-darwin-amd64.tar.gz
robustmq-v0.1.29-darwin-amd64.tar.gz.sha256
robustmq-v0.1.29-darwin-arm64.tar.gz
robustmq-v0.1.29-darwin-arm64.tar.gz.sha256
robustmq-v0.1.29-windows-amd64.tar.gz
robustmq-v0.1.29-windows-amd64.tar.gz.sha256
```

Internal structure of each tar.gz package:
```
robustmq-v0.1.29-linux-amd64/
├── bin/                    # Startup scripts
├── libs/                   # Binary files
│   ├── broker-server
│   ├── cli-command
│   └── cli-bench
├── config/                 # Configuration files
│   ├── version.txt
│   └── server.toml
└── package-info.txt        # Package information
```

## ⚠️ Common Issues

### Permission Error
```bash
Error: GitHub token is required
```
**Solution**: Ensure GITHUB_TOKEN environment variable is set correctly

### Missing Dependencies
```bash
Error: Missing required dependencies: jq
```
**Solution**:
```bash
# macOS
brew install jq

# Ubuntu/Debian
sudo apt install jq

# CentOS/RHEL
sudo yum install jq
```

### Release Already Exists
```bash
Release v0.1.29 already exists, skipping creation
```
**Solution**: Use `--force` parameter to force recreate, or delete existing release first

### Build Failure
```bash
Failed to build packages
```
**Solution**:
1. Check if Rust toolchain is properly installed
2. Ensure all target platforms are installed
3. Check detailed error messages

## 🔍 Debugging Tips

### Use Dry Run Mode
```bash
./scripts/release.sh --dry-run --verbose
```

### View Detailed Logs
```bash
./scripts/release.sh --verbose
```

### Step-by-Step Execution
```bash
# Build packages only, no upload
./scripts/build.sh --all-platforms

# Skip build, upload only
./scripts/release.sh --skip-build
```

### Check GitHub API Response
```bash
# Manually check if release exists
curl -H "Authorization: token $GITHUB_TOKEN" \
     https://api.github.com/repos/robustmq/robustmq/releases/tags/v0.1.29
```

## 📋 Best Practices

1. **Version Management**
   - Follow Semantic Versioning (SemVer)
   - Update version in Cargo.toml before release
   - Create corresponding git tags

2. **Pre-release Checks**
   - Run complete test suite
   - Build and test binaries locally
   - Use --dry-run to preview release operations

3. **Security Considerations**
   - Never hardcode GitHub tokens in code
   - Use environment variables or secure key management systems
   - Rotate GitHub tokens regularly

4. **Automation Integration**
   - Can be integrated into CI/CD pipelines
   - Automatically trigger on merge to main branch
   - Use with GitHub Actions

## 🤝 Contributing

If you need to modify the release script:

1. Make changes in `scripts/release.sh`
2. Update relevant documentation in this file
3. Test all features and options
4. Submit PR with detailed description of changes

---

> **📞 Need Help?**  
> If you have questions, please create an issue in [GitHub Issues](https://github.com/robustmq/robustmq/issues) or contact the development team.