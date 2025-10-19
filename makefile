# Copyright 2023 RobustMQ Team
# Makefile for RobustMQ development and build tasks

VERSION := $(shell grep '^version = ' Cargo.toml | head -n1 | cut -d'"' -f2)

##@ Development
.PHONY: dev
dev: ## Run broker-server in development mode
	cargo run --package cmd --bin broker-server

.PHONY: fmt
fmt: ## Format code with rustfmt
	cargo fmt --all

.PHONY: check
check: ## Quick compilation check
	cargo check --workspace

.PHONY: clippy
clippy: ## Run clippy linter
	cargo clippy --workspace --all-targets --tests -- -D warnings

.PHONY: codecheck
codecheck: ## Run all code quality checks (format, clippy, license)
	@echo "Running code quality checks..."
	hawkeye format
	cargo fmt --all
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets --tests -- -D warnings
	cargo-deny check licenses
	@echo "✅ All checks passed!"

.PHONY: doc
doc: ## Generate documentation
	cargo doc --workspace --no-deps --open

##@ Build
.PHONY: build
build: ## Build debug version for local development
	cargo build --workspace

.PHONY: build-release
build-release: ## Build optimized release version
	cargo build --workspace --release

.PHONY: build-server
build-server: ## Build broker-server binary
	cargo build --package cmd --bin broker-server --release

##@ Test
.PHONY: test
test: ## Run unit tests with cleanup
	@echo "Running unit tests..."
	cargo nextest run --workspace --exclude=robustmq-test --exclude=grpc-clients
	@$(MAKE) clean-test-artifacts

.PHONY: test-all
test-all: ## Run all tests including integration tests
	@echo "Running all tests..."
	cargo nextest run --workspace
	@$(MAKE) clean-test-artifacts

.PHONY: mqtt-ig-test
mqtt-ig-test: ## Run MQTT integration tests
	/bin/bash ./scripts/mqtt-ig.sh
	@$(MAKE) clean-test-artifacts

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

.PHONY: clean-test-artifacts
clean-test-artifacts: ## Clean test artifacts
	@rm -rf target/nextest
	@find target -type f -name "*-????????????????" -delete 2>/dev/null || true
	@find target -name "*test*" -type f -delete 2>/dev/null || true

##@ Install
.PHONY: install
install: ## Install RobustMQ binaries to system
	/bin/bash scripts/install.sh

##@ Help
.PHONY: help
help: ## Display this help message
	@awk 'BEGIN {FS = ":.*##"; printf "\n\033[1mUsage:\033[0m\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

.DEFAULT_GOAL := help
