# Copyright 2023 RobustMQ Team
# Makefile for RobustMQ development and build tasks

VERSION := $(shell grep '^version = ' Cargo.toml | head -n1 | cut -d'"' -f2)

##@ Development
.PHONY: run
run: ## Run broker-server in development mode
	cargo run --package cmd --bin broker-server

.PHONY: codecheck
codecheck: ## Run all code quality checks (format, check, clippy, license, docs)
	@echo "Running code quality checks..."
	hawkeye format
	cargo fmt --all
	cargo fmt --all -- --check
	cargo check --workspace
	cargo clippy --workspace --all-targets --tests -- -D warnings
	cargo-deny check licenses
	@echo "Building documentation..."
	npm run docs:build
	@echo "✅ All checks passed!"

.PHONY: audit
audit: ## Scan dependencies for known vulnerabilities
	@echo "Scanning dependencies for vulnerabilities..."
	cargo audit
	@echo "✅ Audit passed!"

.PHONY: udeps
udeps: ## Check for unused dependencies (requires nightly)
	@echo "Checking for unused dependencies..."
	cargo +nightly udeps --workspace
	@echo "✅ Unused dependency check passed!"

.PHONY: tree
tree: ## Display the dependency tree
	cargo tree

.PHONY: ci-check
ci-check: ## Run comprehensive CI pre-check pipeline (fmt + check + clippy + test + audit)
	@echo "========================================="
	@echo "  RobustMQ CI Pre-Check Pipeline"
	@echo "========================================="
	@echo ""
	@echo "[1/4] Format check..."
	cargo fmt --all -- --check
	@echo "[2/4] Compilation check..."
	cargo check --workspace
	@echo "[3/4] Clippy lint..."
	cargo clippy --workspace --all-targets --tests -- -D warnings
	@echo "[4/4] Security audit..."
	cargo audit
	@echo ""
	@echo "========================================="
	@echo "  ✅ CI pre-check passed!"
	@echo "========================================="

.PHONY: doc
doc: ## Generate documentation
	cargo doc --workspace --no-deps --open

##@ Build
.PHONY: build
build: ## Build current platform package (basic build without frontend)
	@echo "Building current platform package..."
	./scripts/build.sh
	@echo "$(VERSION)" > config/version.ini
	@echo "📝 Wrote version $(VERSION) to config/version.ini"

.PHONY: build-full
build-full: ## Build complete package with frontend (auto-clone frontend repo, build web UI, create tarball)
	@echo "Building complete package with frontend..."
	@echo "This will:"
	@echo "  • Clone robustmq-copilot frontend repository"
	@echo "  • Build web UI with pnpm"
	@echo "  • Compile Rust binaries in release mode"
	@echo "  • Create package: build/robustmq-{version}-{platform}.tar.gz"
	./scripts/build.sh --with-frontend
	@echo "$(VERSION)" > config/version.ini
	@echo "📝 Wrote version $(VERSION) to config/version.ini"

.PHONY: build-version
build-version: ## Build package with specific version (usage: make build-version VERSION=v0.1.30)
	@echo "Building package with version: $(VERSION)"
	./scripts/build.sh --version $(VERSION)

##@ Release
.PHONY: release
release: ## Create new GitHub release and upload package
	@echo "Creating new GitHub release..."
	@echo "This will:"
	@echo "  • Create GitHub release with current version"
	@echo "  • Build and upload package for current platform"
	@echo "  • Requires GITHUB_TOKEN environment variable"
	./scripts/release.sh

.PHONY: release-docker
release-docker: ## Build and push application image to GHCR
	@echo "Building application image for GHCR (org=robustmq, version=$(VERSION))..."
	./scripts/build-and-push-app.sh --org robustmq --version $(VERSION) --registry ghcr --push-latest

.PHONY: release-version
release-version: ## Create new GitHub release with specific version (usage: make release-version VERSION=v0.1.30)
	@echo "Creating GitHub release with version: $(VERSION)"
	./scripts/release.sh --version $(VERSION)

##@ Test
.PHONY: test
test: ## Run unit tests with cleanup
	@echo "Running unit tests..."
	cargo nextest run --workspace \
		--exclude=robustmq-test \
		--exclude=grpc-clients \
		--filter-expr '!(test(meta) & package(storage-adapter))'

## Integration tests are split by protocol so each can be diagnosed
## independently in CI (its own 3-node cluster, its own pass/fail): core
## (protocol-agnostic: engine/meta/grpc-clients/mcp/etc.), mqtt, nats, mq9,
## kafka, amqp. Each has a plain (assumes a broker is already running) and a
## -ci (starts/stops its own 3-node cluster) variant.
.PHONY: ig-test-core ig-test-core-ci
ig-test-core: ## Run core (protocol-agnostic) integration tests (assumes broker is already running)
	/bin/bash ./scripts/ig-test.sh core
ig-test-core-ci: ## Run core integration tests with broker startup (for CI)
	/bin/bash ./scripts/ig-test.sh core --start-broker

.PHONY: ig-test-mqtt ig-test-mqtt-ci
ig-test-mqtt: ## Run MQTT integration tests (assumes broker is already running)
	/bin/bash ./scripts/ig-test.sh mqtt
ig-test-mqtt-ci: ## Run MQTT integration tests with broker startup (for CI)
	/bin/bash ./scripts/ig-test.sh mqtt --start-broker

.PHONY: ig-test-nats ig-test-nats-ci
ig-test-nats: ## Run NATS integration tests (assumes broker is already running)
	/bin/bash ./scripts/ig-test.sh nats
ig-test-nats-ci: ## Run NATS integration tests with broker startup (for CI)
	/bin/bash ./scripts/ig-test.sh nats --start-broker

.PHONY: ig-test-mq9 ig-test-mq9-ci
ig-test-mq9: ## Run MQ9 integration tests (assumes broker is already running)
	/bin/bash ./scripts/ig-test.sh mq9
ig-test-mq9-ci: ## Run MQ9 integration tests with broker startup (for CI)
	/bin/bash ./scripts/ig-test.sh mq9 --start-broker

.PHONY: ig-test-kafka ig-test-kafka-ci
ig-test-kafka: ## Run Kafka integration tests, incl. Java client (assumes broker is already running)
	/bin/bash ./scripts/ig-test.sh kafka
ig-test-kafka-ci: ## Run Kafka integration tests with broker startup (for CI)
	/bin/bash ./scripts/ig-test.sh kafka --start-broker

.PHONY: ig-test-amqp ig-test-amqp-ci
ig-test-amqp: ## Run AMQP integration tests, incl. RabbitMQ Java client (assumes broker is already running)
	/bin/bash ./scripts/ig-test.sh amqp
ig-test-amqp-ci: ## Run AMQP integration tests with broker startup (for CI)
	/bin/bash ./scripts/ig-test.sh amqp --start-broker

.PHONY: ig-test-all
ig-test-all: ## Run every integration test suite sequentially (assumes broker is already running)
	/bin/bash ./scripts/ig-test.sh core
	/bin/bash ./scripts/ig-test.sh mqtt
	/bin/bash ./scripts/ig-test.sh nats
	/bin/bash ./scripts/ig-test.sh mq9
	/bin/bash ./scripts/ig-test.sh kafka
	/bin/bash ./scripts/ig-test.sh amqp

.PHONY: kafka-test
kafka-test: ## Run Kafka Java-client integration tests (assumes broker is already running)
	@echo "Running Kafka integration tests (broker must be running)..."
	cd tests/kafka-java && mvn test \
		$(if $(KAFKA_CLIENTS_VERSION),-Dkafka.clients.version=$(KAFKA_CLIENTS_VERSION),)

.PHONY: rabbitmq-test
rabbitmq-test: ## Run RabbitMQ Java-client integration tests (assumes broker is already running)
	@echo "Running RabbitMQ integration tests (broker must be running)..."
	cd tests/rabbitmq-java && mvn test \
		$(if $(RABBITMQ_CLIENT_VERSION),-Drabbitmq.client.version=$(RABBITMQ_CLIENT_VERSION),)

##@ Clean
.PHONY: clean
clean: ## Clean all build artifacts
	cargo clean
	rm -rf build

##@ Help
.PHONY: help
help: ## Display this help message
	@awk 'BEGIN {FS = ":.*##"; printf "\n\033[1mUsage:\033[0m\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

.DEFAULT_GOAL := help
