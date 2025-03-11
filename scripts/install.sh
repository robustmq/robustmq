# Copyright 2023 RobustMQ Team
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


set -u -e

OS_TYPE=
ARCH_TYPE=
VERSION=${1:-latest}
GITHUB_ORG=robustmq
GITHUB_REPO=robustmq
BIN=robustmq

get_os_type() {
    os_type="$(uname -s)"

    case "$os_type" in
    Darwin)
        OS_TYPE=darwin
        ;;
    Linux)
        OS_TYPE=linux
        ;;
    *)
        echo "Error: Unknown OS type: $os_type"
        exit 1
    esac
}

get_arch_type() {
    arch_type="$(uname -m)"

    case "$arch_type" in
    arm64)
        ARCH_TYPE=arm64
        ;;
    aarch64)
        ARCH_TYPE=arm64
        ;;
    x86_64)
        ARCH_TYPE=amd64
        ;;
    amd64)
        ARCH_TYPE=amd64
        ;;
    *)
        echo "Error: Unknown CPU type: $arch_type"
        exit 1
    esac
}

get_os_type
get_arch_type

if [ -n "${OS_TYPE}" ] && [ -n "${ARCH_TYPE}" ]; then
    # Use the latest nightly version.
    if [ "${VERSION}" = "latest" ]; then
        VERSION=$(curl -s -XGET "https://api.github.com/repos/${GITHUB_ORG}/${GITHUB_REPO}/releases" | grep tag_name | grep nightly | cut -d: -f 2 | sed 's/.*"\(.*\)".*/\1/' | uniq | sort -r | head -n 1)
        if [ -z "${VERSION}" ]; then
            echo "Failed to get the latest version."
            exit 1
        fi
    fi

    echo "Downloading ${BIN}, OS: ${OS_TYPE}, Arch: ${ARCH_TYPE}, Version: ${VERSION}"
    PACKAGE_NAME="${BIN}-${OS_TYPE}-${ARCH_TYPE}-${VERSION}.tar.gz"

    if [ -n "${PACKAGE_NAME}" ]; then
      wget "https://github.com/${GITHUB_ORG}/${GITHUB_REPO}/releases/download/${VERSION}/${PACKAGE_NAME}"

      # Extract the binary and clean the rest.
      tar xvf "${PACKAGE_NAME}" && \
      mv "${PACKAGE_NAME%.tar.gz}/${BIN}" "${PWD}" && \
      rm -r "${PACKAGE_NAME}" && \
      rm -r "${PACKAGE_NAME%.tar.gz}" && \
      echo "Run './${BIN} --help' to get started"
    fi
fi
