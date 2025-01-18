## Project packaging
RobustMQ can be packaged with the Make command that comes with the project itself.
```
Build
  build                           Build mac version robustmq.
  build-mac-release               Build mac version robustmq.
  build-linux-release             Build linux version robustmq.
  build-win-release               Build win version robustmq.
  build-arm-release               Build arm version robustmq.
```

## Bundling local versions
Automatically identify the current machine model for packaging.
```
make build
```
## Packaging the Mac version
Package versions for both the x86_64-apple-darwi and aarch64-apple-darwin platforms.
```
make build-mac-release
```
## Bundling Linux versions
Package aarch64-unknown-linux-gnu and aarch64-unknown-linux-musl versions.
```
make build-linux-release  
```
## Packaging the Win build
Package x86_64-pc-windows-gnu and i686-pc-windows-gnu versions for both platforms.
```
make build-win-release
```
## Packaging Arm versions
Package aarch64-pc-windows-gnullvm platform version.
```
make build-arm-release
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