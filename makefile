# The arguments for building images.
VERSION:=$(shell grep '^version = ' Cargo.toml | head -n1 | cut -d'"' -f2)

##@ Build
.PHONY: build
build: ## Build local machine version robustmq.
	sh scripts/build-release.sh local $(VERSION)

# MacOS
.PHONY: build-mac-x86_64-release
build-mac-x86_64-release: ## Build mac x86_64 version robustmq.
	sh scripts/build-release.sh mac-x86_64 $(VERSION)

.PHONY: build-mac-arm64-release
build-mac-arm64-release: ## Build mac arm64 version robustmq.
	sh scripts/build-release.sh mac-arm64 $(VERSION)

# Linux
.PHONY: build-linux-x86_64-release
build-linux-x86_64-release: ## Build linux x86_64 version robustmq.
	sh scripts/build-release.sh linux-x86_64 $(VERSION)

.PHONY: build-linux-arm64-release
build-linux-arm64-release: ## Build linux arm64 version robustmq.
	sh scripts/build-release.sh linux-arm64 $(VERSION)

# Windows
.PHONY: build-win-x86_64-release
build-win-x86_64-release: ## Build windows x86 64bit version robustmq.
	sh scripts/build-release.sh win-x86_64 $(VERSION)

.PHONY: build-win-x86-release
build-win-x86-release: ## Build windows x86 32bit version robustmq.
	sh scripts/build-release.sh win-x86 $(VERSION)

.PHONY: build-win-arm64-release
build-win-arm64-release: ## Build windows arm64 version robustmq.
	sh scripts/build-release.sh win-arm64 $(VERSION)

##@ Test
.PHONY: test
test:  ## Unit testing for Robustmq
	sh ./scripts/unit-test.sh dev

.PHONY: mqtt-ig-test
mqtt-ig-test:  ## Integration testing for MQTT Broker
	sh ./scripts/mqtt-ig-test.sh dev

.PHONY: place-ig-test
place-ig-test:  ## Integration testing for Placement Center
	sh ./scripts/place-ig-test.sh dev

.PHONY: journal-ig-test
journal-ig-test:  ## Integration testing for Journal Engine
	sh ./scripts/journal-ig-test.sh dev

##@ Other
.PHONY: clean
clean:  ## Clean the project.
	cargo clean
	rm -rf build

.PHONY: help
help: ## Display help messages.
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  \033[36m%-30s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)
