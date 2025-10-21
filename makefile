# Copyright 2023 RobustMQ Team
# Makefile for RobustMQ development and build tasks

VERSION := $(shell grep '^version = ' Cargo.toml | head -n1 | cut -d'"' -f2)

##@ Development
.PHONY: dev
dev: ## Run broker-server in development mode
	cargo run --package cmd --bin broker-server

.PHONY: codecheck
codecheck: ## Run all code quality checks (format, check, clippy, license)
	@echo "Running code quality checks..."
	hawkeye format
	cargo fmt --all
	cargo fmt --all -- --check
	cargo check --workspace
	cargo clippy --workspace --all-targets --tests -- -D warnings
	cargo-deny check licenses
	@echo "✅ All checks passed!"

.PHONY: doc
doc: ## Generate documentation
	cargo doc --workspace --no-deps --open

##@ Build
.PHONY: build
build: ## Build current platform package (basic build without frontend)
	@echo "Building current platform package..."
	./scripts/build.sh

.PHONY: build-full
build-full: ## Build complete package with frontend (auto-clone frontend repo, build web UI, create tarball)
	@echo "Building complete package with frontend..."
	@echo "This will:"
	@echo "  • Clone robustmq-copilot frontend repository"
	@echo "  • Build web UI with pnpm"
	@echo "  • Compile Rust binaries in release mode"
	@echo "  • Create package: build/robustmq-{version}-{platform}.tar.gz"
	./scripts/build.sh --with-frontend

.PHONY: build-version
build-version: ## Build package with specific version (usage: make build-version VERSION=v0.1.30)
	@echo "Building package with version: $(VERSION)"
	./scripts/build.sh --version $(VERSION)

.PHONY: build-clean
build-clean: ## Clean build directory and rebuild package (removes all build artifacts first)
	@echo "Cleaning and rebuilding package..."
	@echo "This will:"
	@echo "  • Remove build/ directory completely"
	@echo "  • Rebuild everything from scratch"
	@echo "  • Create package: build/robustmq-{version}-{platform}.tar.gz"
	./scripts/build.sh --clean

##@ Docker
.PHONY: docker-deps
docker-deps: ## Build and push dependency base image to GHCR (for CI/CD optimization)
	@echo "Building dependency base image..."
	@echo "This will:"
	@echo "  • Build Rust dependency cache image (~8-10GB)"
	@echo "  • Push to ghcr.io/socutes/robustmq/rust-deps:latest"
	@echo "  • Requires Docker and GHCR login"
	@echo "  • Takes 20-40 minutes on first build"
	./scripts/build-and-push-deps.sh

.PHONY: docker-deps-tag
docker-deps-tag: ## Build and push dependency image with specific tag (usage: make docker-deps-tag TAG=2025-10-20)
	@echo "Building dependency image with tag: $(TAG)"
	./scripts/build-and-push-deps.sh $(TAG)

.PHONY: docker-app
docker-app: ## Build and push application image to registry (requires ARGS parameter)
	@echo "Building application image..."
	@echo "This will:"
	@echo "  • Build RobustMQ application Docker image"
	@echo "  • Push to specified registry (GHCR or Docker Hub)"
	@echo "  • Usage: make docker-app ARGS='--org yourorg --version 0.2.0'"
	@echo "  • Example: make docker-app ARGS='--org socutes --version 0.2.0 --registry ghcr'"
	./scripts/build-and-push-app.sh $(ARGS)

.PHONY: docker-app-ghcr
docker-app-ghcr: ## Build and push application image to GHCR (usage: make docker-app-ghcr ORG=yourorg VERSION=0.2.0)
	@echo "Building application image for GHCR..."
	./scripts/build-and-push-app.sh --org $(ORG) --version $(VERSION) --registry ghcr --push-latest

.PHONY: docker-app-dockerhub
docker-app-dockerhub: ## Build and push application image to Docker Hub (usage: make docker-app-dockerhub ORG=yourorg VERSION=0.2.0)
	@echo "Building application image for Docker Hub..."
	./scripts/build-and-push-app.sh --org $(ORG) --version $(VERSION) --registry dockerhub

##@ Release
.PHONY: release
release: ## Create new GitHub release and upload package
	@echo "Creating new GitHub release..."
	@echo "This will:"
	@echo "  • Create GitHub release with current version"
	@echo "  • Build and upload package for current platform"
	@echo "  • Requires GITHUB_TOKEN environment variable"
	./scripts/release.sh

.PHONY: release-version
release-version: ## Create new GitHub release with specific version (usage: make release-version VERSION=v0.1.30)
	@echo "Creating GitHub release with version: $(VERSION)"
	./scripts/release.sh --version $(VERSION)

.PHONY: release-upload
release-upload: ## Upload package to existing release (usage: make release-upload VERSION=v0.1.30)
	@echo "Uploading package to existing release: $(VERSION)"
	./scripts/release.sh --upload-only --version $(VERSION)

##@ Install
.PHONY: install
install: ## Auto-download and install RobustMQ
	@echo "Installing RobustMQ..."
	./scripts/install.sh

##@ Test
.PHONY: test
test: ## Run unit tests with cleanup
	@echo "Running unit tests..."
	cargo nextest run --workspace \
		--exclude=robustmq-test \
		--exclude=grpc-clients \
		--filter-expr '!(test(meta) & package(storage-adapter))'

.PHONY: ig-test
ig-test: ## Run MQTT integration tests
	/bin/bash ./scripts/ig-test.sh

##@ Clean
.PHONY: clean
clean: ## Full clean (removes all build artifacts)
	cargo clean
	rm -rf build

.PHONY: clean-light
clean-light: ## Light clean (cache and test artifacts only)
	@echo "Light cleaning..."
	@rm -rf target/*/incremental
	@rm -rf target/nextest
	@rm -rf target/debug/build
	@find target -type f -name "*-????????????????" -delete 2>/dev/null || true
	@echo "✅ Done!"

##@ Help
.PHONY: help
help: ## Display this help message
	@awk 'BEGIN {FS = ":.*##"; printf "\n\033[1mUsage:\033[0m\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

.DEFAULT_GOAL := help
