# The arguments for building images.
VERSION:=$(shell grep "version =" Cargo.toml | awk -F'"' '{print $2}' | head -n 1 | sed 's/version = //g')

##@ Build Mac Release
.PHONY: build
build: ## Build mac version robustmq.
	sh scripts/build-release.sh local $(VERSION)

##@ Build Mac Release
.PHONY: build-mac-release
build-mac-release: ## Build mac version robustmq.
	sh scripts/build-release.sh mac $(VERSION)

##@ Build Linux Release
.PHONY: build-linux-release
build-linux-release: ## Build linux version robustmq.
	sh scripts/build-release.sh linux $(VERSION)

##@ Build Win Release
.PHONY: build-win-release
build-win-release: ## Build win version robustmq.
	sh scripts/build-release.sh win $(VERSION)

##@ Build Arm Release
.PHONY: build-arm-release
build-arm-release: ## Build arm version robustmq.
	sh scripts/build-release.sh arm $(VERSION)

.PHONY: test
test:  ## Unit testing for Robustmq
	sh ./scripts/unit-test.sh dev

.PHONY: test-ci
test-ci:  ## Unit testing for Robustmq
	sh ./scripts/unit-test.sh ci

.PHONY: mqtt-ig-test
mqtt-ig-test:  ## Integration testing for MQTT Broker
	sh ./scripts/mqtt-ig-test.sh dev

.PHONY: mqtt-ig-test-ci
mqtt-ig-test-ci:  ## Integration testing for MQTT Broker
	sh ./scripts/mqtt-ig-test.sh ci

.PHONY: place-ig-test
place-ig-test:  ## Integration testing for Placement Center
	sh ./scripts/place-ig-test.sh dev

.PHONY: place-ig-test-ci
place-ig-test-ci:  ## Integration testing for Placement Center
	sh ./scripts/place-ig-test.sh ci

.PHONY: journal-ig-test
journal-ig-test:  ## Integration testing for Journal Engine
	sh ./scripts/journal-ig-test.sh dev

.PHONY: journal-ig-test
journal-ig-test-ci:  ## Integration testing for Journal Engine
	sh ./scripts/journal-ig-test.sh ci

.PHONY: clean
clean:  ## Clean the project.
	cargo clean
	rm -rf build

.PHONY: help
help: ## Display help messages.
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  \033[36m%-30s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)
