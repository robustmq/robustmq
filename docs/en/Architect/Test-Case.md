# Test Case
## Installing cargo-nextest
The integration tests of RobustMQ rely on cargo-nextest to speed up the execution of test cases, so you need to install cargo-nextest when running integration tests.

For installation instructions, please refer to this document: [Pre-built binaries - cargo-nextest](https://nexte.st/docs/installation/pre-built-binaries/)

- Using cargo-binstall:
```
cargo binstall cargo-nextest --secure
```

- Linux installation:
```
curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin
```

- Linux aarch64 installation:
```
curl -LsSf https://get.nexte.st/latest/linux-arm | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin
```

- macOS installation:
```
curl -LsSf https://get.nexte.st/latest/mac | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin
```

- Windows installation:
```
curl -LsSf https://get.nexte.st/latest/windows-tar | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin
```

## Running Tests

Enter the corresponding make command in the root directory.

- Unit tests

  All test unit test cases:
  ```
  make test
  ```

- Integration tests

  MQTT Broker:
  ```
  make mqtt-ig-test
  ```

  Placement Center:
  ```
  make place-ig-test
  ```

  Journal Engine:
  ```
  make journal-ig-test
  ```
