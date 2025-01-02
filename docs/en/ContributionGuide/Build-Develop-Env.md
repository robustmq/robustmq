# Setting Up the Basic Development Environment
## Build in Mac
### Overview
> ⚠️Note: The current project already includes a `rust-toolchain.toml` file,
> which means it will use the Rust environment configured in this file by default. If Rust is installed via Homebrew,
> there may be issues with version overrides, and it will be necessary to uninstall it and rebuild the environment according to these instructions.

To run the code, you first need to set up the Rust development environment. After initializing the Rust environment, the project mainly depends on cmake, rocksdb,
protoc, and you need to install these dependencies based on your operating system environment.

- Installing Rust Basic Environment
  Refer to the documentation: [Installing Rust Environment - Rust Language Bible (Rust Course)](https://course.rs/first-try/installation.html)

- Rust Version
  The Rust version currently depended on is: nightly-2024-11-08
  ```shell
  rustup install nightly-2024-11-08
  rustup default nightly-2024-11-08
  rustc --version
  ```

- Installing Cmake.
  The installation command for Mac is as follows:
  ```shell
  brew install cmake
  ```

- Installing RocksDB
  Refer to the documentation: [GitHub - rust-rocksdb/rust-rocksdb: rust wrapper for rocksdb](https://github.com/rust-rocksdb/rust-rocksdb) to install rocksdb.
  The installation command for Mac is as follows:
  ```shell
  brew install rocksdb
  ```

- Installing protoc
  Refer to the documentation: [mac installation of proto and simple compilation to dart files](https://www.jianshu.com/p/341293ee1286) to install protoc.
  The installation command for Mac is as follows:
  ```shell
  brew install protobuf
  ```

### Developing pre-commit Plugins

RobustMQ uses `pre-commit` for pre-submission code checks by default, so you need to install a specific version of the `pre-commit` tool.

First, you need to create a virtual environment using `Python` (version 3.8 or above), and the installation command is as follows:
```shell
python3 -m venv precommit_venv
```

After installation, enter the virtual environment with the following command:
```shell
source ./precommit_venv/bin/activate
```

Then, install the corresponding version of the `pre-commit` tool, here using the version of `pre-commit` specified in the project environment:
```shell
pip3 install -r ./.requirements-precommit.txt
```

After the installation is complete, you need to initialize the `pre-commit` hooks for the project content with the following command:
```shell
pre-commit install
```

> ⚠️Note: Subsequent `git commit` operations will execute the corresponding checks with `pre-commit`. If you do not want to check every time you commit, you can skip the check using `git commit -n`.

The `pre-commit` checks use tools such as next-test, hawkeye, clippy, typos, cargo-deny, etc. If these tools are missing, you can refer to the following commands for installation:
```shell
cargo install hawkeye@5.8.1
cargo install typos-cli
cargo install cargo-deny@0.16.2 --locked
cargo install cargo-nextest@0.9.84
```
