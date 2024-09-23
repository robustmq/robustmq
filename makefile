# The arguments for building images.
TARGET = robustmq
BUILD_FOLD = ./build
VERSION:=$(shell grep "version =" Cargo.toml | awk -F'"' '{print $2}' | head -n 1 | sed 's/version = //g')
PACKAGE_FOLD_NAME = ${TARGET}-$(VERSION)

##@ Build Mac Release
.PHONY: build-mac-release
build-mac-release: ## Build debug version robustmq.
	mkdir -p ${BUILD_FOLD}
	cargo build --release
	mkdir -p $(BUILD_FOLD)/${PACKAGE_FOLD_NAME}
	mkdir -p $(BUILD_FOLD)/${PACKAGE_FOLD_NAME}/bin
	mkdir -p $(BUILD_FOLD)/${PACKAGE_FOLD_NAME}/libs
	mkdir -p $(BUILD_FOLD)/${PACKAGE_FOLD_NAME}/config
	cp -rf target/release/mqtt-server $(BUILD_FOLD)/${PACKAGE_FOLD_NAME}/libs 
	cp -rf target/release/placement-center $(BUILD_FOLD)/${PACKAGE_FOLD_NAME}/libs 
	cp -rf target/release/journal-server $(BUILD_FOLD)/${PACKAGE_FOLD_NAME}/libs 
	cp -rf target/release/cli-command-mqtt $(BUILD_FOLD)/${PACKAGE_FOLD_NAME}/libs 
	cp -rf target/release/cli-command-placement $(BUILD_FOLD)/${PACKAGE_FOLD_NAME}/libs 
	cp -rf bin/* $(BUILD_FOLD)/${PACKAGE_FOLD_NAME}/bin
	cp -rf config/* $(BUILD_FOLD)/${PACKAGE_FOLD_NAME}/config
	chmod -R 777 $(BUILD_FOLD)/${PACKAGE_FOLD_NAME}/bin/*
	cd $(BUILD_FOLD) && tar zcvf ${PACKAGE_FOLD_NAME}.tar.gz ${PACKAGE_FOLD_NAME} && rm -rf ${PACKAGE_FOLD_NAME}
	echo "build release package success. ${PACKAGE_FOLD_NAME}.tar.gz "

.PHONY: test
test:  ## Integration testing for Robustmq
	sh ./scripts/integration-testing.sh

.PHONY: clean
clean:  ## Clean the project.
	cargo clean
	rm -rf build

.PHONY: help
help: ## Display help messages.
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  \033[36m%-30s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)
