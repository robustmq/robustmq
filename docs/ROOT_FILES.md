# Root Directory Files Documentation

This document explains the purpose of each file in the root directory.

## Core Project Files

| File | Purpose | Required |
|------|---------|----------|
| `Cargo.toml` | Rust workspace configuration | ✅ Yes |
| `Cargo.lock` | Dependency version lock file | ✅ Yes |
| `LICENSE` | Apache 2.0 License | ✅ Yes |
| `README.md` | Project documentation | ✅ Yes |
| `makefile` | Build and development commands | ✅ Yes |

## Rust Tooling

| File | Purpose | Required |
|------|---------|----------|
| `rust-toolchain.toml` | Rust version pinning | ✅ Yes |
| `rustfmt.toml` | Code formatting rules | ✅ Yes |
| `deny.toml` | Dependency license/security checks | ✅ Yes |
| `_typos.toml` | Spell checking configuration | ✅ Yes |
| `licenserc.toml` | License header validation | ✅ Yes |
| `cliff.toml` | Changelog generation | ✅ Yes |

## Git Configuration

| File | Purpose | Required |
|------|---------|----------|
| `.gitignore` | Git ignore patterns | ✅ Yes |
| `.gitattributes` | Git file attributes (line endings, etc.) | ✅ Yes |

## Development Tools

| File | Purpose | Required |
|------|---------|----------|
| `.pre-commit-config.yaml` | Pre-commit hooks configuration | 🟡 Optional but recommended |
| `.requirements-precommit.txt` | Python dependencies for pre-commit | 🟡 Optional |
| `mirror` | Cargo mirror configuration (China) | 🟡 Optional |

## Documentation

| File | Purpose | Required |
|------|---------|----------|
| `package.json` | Node.js dependencies for VitePress docs | ✅ Yes (for docs) |
| `package-lock.json` | npm lock file | ✅ Yes (for docs) |

## Nix Package Manager (Optional)

| File | Purpose | Required |
|------|---------|----------|
| `flake.nix` | Nix flake configuration | 🟢 Optional |
| `flake.lock` | Nix dependencies lock | 🟢 Optional |
| `shell.nix` | Nix development shell | 🟢 Optional |

> **Note**: Nix files are for developers who use Nix package manager for reproducible development environments. They can be safely ignored if you don't use Nix.

## Quick Reference

### For New Contributors

Required files to understand:
1. `README.md` - Start here
2. `Cargo.toml` - Project structure
3. `makefile` - Available commands
4. `.pre-commit-config.yaml` - Code quality checks

### For Maintainers

Files to update when:
- **Changing Rust version**: Update `rust-toolchain.toml`
- **Adding dependencies**: Update `Cargo.toml`
- **Changing formatting**: Update `rustfmt.toml`
- **Changing license policy**: Update `deny.toml`
- **Updating docs dependencies**: Update `package.json`

## Makefile Commands

```bash
# Code quality checks
make codecheck              # Run all code quality checks

# Build
make build                  # Build local version
make build-mac-arm64-release  # Build for macOS ARM64

# Test
make test                   # Run unit tests
make mqtt-ig-test           # Run MQTT integration tests

# Clean
make clean                  # Full clean
make clean-incremental      # Clean cache only
make clean-light            # Light clean

# Help
make help                   # Show all commands
```

## Pre-commit Setup

To set up pre-commit hooks:

```bash
# Install pre-commit
pip install -r .requirements-precommit.txt

# Install hooks
pre-commit install

# Run manually
pre-commit run --all-files
```

## File Not Needed

The following files should NOT be in the repository:
- `.DS_Store` - macOS system files (already in .gitignore)
- `*.swp`, `*.swo` - Editor temporary files (already in .gitignore)
- `target/` - Rust build artifacts (already in .gitignore)
- `node_modules/` - npm dependencies (already in .gitignore)

