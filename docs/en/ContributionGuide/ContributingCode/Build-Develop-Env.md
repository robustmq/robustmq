# Setting Up the Basic Development Environment

# Setting Up the Mac Environment

## Main Steps

::: tip
⚠️ **Note**: The project currently includes a `rust-toolchain.toml` file by default, so it will use the Rust environment configured in that file. If you have installed Rust via Homebrew, you may encounter version conflicts. You will need to uninstall Rust and then set up the environment according to the instructions below.
:::

To run the code, you need to set up the Rust development environment first. After initializing the Rust environment, the project mainly depends on `cmake`, `rocksdb`, and `protoc`. You need to install these dependencies based on your operating system.

- **Install Rust Basic Environment**
 - Reference Document: [https://course.rs/first-try/installation.html](https://course.rs/first-try/installation.html)

- **Rust Version**
 - The current Rust version required is `stable`.
  ```shell
  rustup install stable
  rustup default stable
  rustc --version
  ```

- **Install Cmake**
 - Installation command for macOS:
  ```shell
  brew install cmake
  ```

- **Install RocksDB**
 - Reference Document: [https://github.com/rust-rocksdb/rust-rocksdb](https://github.com/rust-rocksdb/rust-rocksdb)
 - Installation command for macOS:
  ```shell
  brew install rocksdb
  ```

- **Install Protoc**
 - Reference Document: [https://www.jianshu.com/p/341293ee1286](https://www.jianshu.com/p/341293ee1286)
 - Installation command for macOS:
  ```shell
  brew install protobuf
  ```

### Configuring the pre-commit Plugin

RobustMQ uses `pre-commit` by default for code pre-commit checks, so you need to install the specified version of the `pre-commit` tool.

First, you need to create a virtual environment using `Python` (version 3.8 or higher). The installation command is as follows:
```shell
python3 -m venv precommit_venv
```

After installation, activate the virtual environment with the following command:
```shell
source ./precommit_venv/bin/activate
```

Then, install the specified version of `pre-commit` using the project's environment:
```shell
pip3 install -r ./.requirements-precommit.txt
```

After installation, initialize the `pre-commit` hooks for the project content with the following command:
```shell
pre-commit install
```

::: tip
⚠️ **Note**: Subsequent `git commit` operations will trigger the corresponding checks by `pre-commit`. If you do not want to perform checks with each commit, you can use `git commit -n` to skip the checks.
:::

The checks carried by `pre-commit` use tools such as `next-test`, `hawkeye`, `clippy`, `typos`, and `cargo-deny`. If any of these tools are missing, you can install them using the following commands:
```shell
cargo install hawkeye@5.8.1
cargo install typos-cli
cargo install cargo-deny@0.16.2 --locked
cargo install cargo-nextest@0.9.84
```
