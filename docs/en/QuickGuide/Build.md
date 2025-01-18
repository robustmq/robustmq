## 项目打包
可以通过项目自身携带的 Make 命令打包。
```
Build
  build                           Build mac version robustmq.
  build-mac-release               Build mac version robustmq.
  build-linux-release             Build linux version robustmq.
  build-win-release               Build win version robustmq.
  build-arm-release               Build arm version robustmq.
```

## 打包本地版本
```
make build
```
## 打包 Mac 版本
```
make build-mac-release
```
## 打包 Linux 版本
```
make build-linux-release  
```
## 打包 Win 版本
```
make build-win-release
```
## 打包 Arm 版本
```
make build-arm-release
```

## 产物
二进制安装包位于 build 目录下：
```
$ tree build/
build/
├── robustmq-0.1.6.tar.gz
```
解压后结构如下：
```
$ tree robustmq-0.1.6
robustmq-0.1.6
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