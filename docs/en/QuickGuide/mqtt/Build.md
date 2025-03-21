## Project packaging

RobustMQ can be packaged with the Make command that comes with the project itself.

```
Build
  build                           Build local machine version robustmq.
  build-mac-x86_64-release       Build mac x86_64 version robustmq.
  build-mac-arm64-release        Build mac arm64 version robustmq.
  build-linux-x86_64-release     Build linux x86_64 version robustmq.
  build-linux-arm6a4-release      Build linux arm64 version robustmq.
  build-win-x86_64-release       Build windows x86 64bit version robustmq.
  build-win-x86-release          Build windows x86 32bit version robustmq.
  build-win-arm64-release        Build windows arm64 version robustmq.
```

## Bundling local version

Automatically identify the current machine model for packaging.

- Note: Local development versions, without any optimizations, are only used for local development and debugging.

```shell
make build
```

## Packaging the Mac versions

#### Packaging x86_64-apple-darwin platform version.

```shell
make build-mac-x86_64-release
```

#### Packaging aarch64-apple-darwin platform version.

```shell
make build-mac-arm64-release
```

## Bundling Linux versions

#### Packaging x86_64-unknown-linux-gnu and x86_64-unknown-linux-musl versions.

```shell
make build-linux-x86_64-release
```

#### Packaging aarch64-unknown-linux-gnu and aarch64-unknown-linux-musl versions.

```shell
make build-linux-arm64-release
```

## Packaging the Windows versions

#### Packaging x86_64-pc-windows-gnu version.

```shell
make build-win-x86_64-release
```

#### Packaging i686-pc-windows-gnu version.

```shell
make build-win-x86-release
```

#### Packaging aarch64-pc-windows-gnullvm version.

```shell
make build-win-arm64-release
```

## Packaged product

The resulting binary installation is located in the build directory:

```
$ tree build/
build/
├── robustmq-0.1.6.tar.gz
```

After decompression, the structure is as follows:

```
$ tree robustmq-0.1.6
robustmq-0.1.6
├── bin #  Executable file directory
│   ├── robust-ctl  # RobustMQ Command entry file
│   └── robust-server #  RobustMQ Server entry file
├── config # Configuration file directory
│   ├── example
│   │   ├── certs
│   │   │   ├── ca.pem
│   │   │   ├── cert.pem
│   │   │   └── key.pem
│   │   ├── log4rs.yaml.example
│   │   └── mqtt-server.toml.example
│   ├── journal-server.toml
│   ├── log-config
│   │   └── mqtt-log4rs.yaml
│   ├── log4rs.yaml
│   ├── mqtt-server.toml
│   └── placement-center.toml
└── libs # relevant Lib file directory
    ├── cli-command
    ├── journal-server
    ├── mqtt-server
    └── placement-center
```
