## 项目打包

RobustMQ 可以通过项目自带的 Make 命令打包。

```
Build
  build                          Build local machine version robustmq.
  build-mac-x86_64-release       Build mac x86_64 version robustmq.
  build-mac-arm64-release        Build mac arm64 version robustmq.
  build-linux-x86_64-release     Build linux x86_64 version robustmq.
  build-linux-arm64-release      Build linux arm64 version robustmq.
  build-win-x86_64-release       Build windows x86 64bit version robustmq.
  build-win-x86-release          Build windows x86 32bit version robustmq.
  build-win-arm64-release        Build windows arm64 version robustmq.
```

## 打包本地开发版本

会自动识别当前机器型号，并进行打包。

- 注意：本地开发版本，不包含任何优化，仅用于本地开发调试。

```shell
make build
```

## 打包 Mac 版本

#### 打包 x86_64-apple-darwin 平台的版本。

```shell
make build-mac-x86_64-release
```

#### 打包 aarch64-apple-darwin 平台的版本。

```shell
make build-mac-arm64-release
```

## 打包 Linux 版本

#### 打包 x86_64-unknown-linux-gnu 和 x86_64-unknown-linux-musl 两个平台的版本。

```shell
make build-linux-x86_64-release
```

#### 打包 aarch64-unknown-linux-gnu 和 aarch64-unknown-linux-musl 两个平台的版本。

```shell
make build-linux-arm64-release
```

## 打包 Windows 版本

#### 打包 x86_64-pc-windows-gnu 平台的版本。

```shell
make build-win-x86_64-release
```

#### 打包 i686-pc-windows-gnu 平台的版本。

```shell
make build-win-x86-release
```

#### 打包 aarch64-pc-windows-gnullvm 平台的版本。

```shell
make build-win-arm64-release
```

## 产物

打包生成后的二进制安装包位于 build 目录下：

```shell
$ tree build/
build/
├── robustmq-0.1.14.tar.gz
```

解压后结构如下：

```shell
$ tree robustmq-0.1.14
robustmq-0.1.14
├── bin #  可执行文件目录
│   ├── robust-ctl  # RobustMQ Command 入口文件
│   └── robust-server # RobustMQ Server 入口文件
├── config # 配置文件目录
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
└── libs # 相关Lib文件目录
    ├── cli-command
    ├── journal-server
    ├── mqtt-server
    └── placement-center
```
