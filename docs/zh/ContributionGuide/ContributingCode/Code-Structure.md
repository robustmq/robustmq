# 代码结构
## 根目录

主要关注： bin、config、docs、makefile、scripts、src、tests 这几个文件和目录.
```
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── README.md
├── benches  # 存放压测相关文件的目录
├── bin # 存放启动服务的脚本文件的目录
├── build.rs # Cargo 的build.rs文件
├── config # 存放项目配置的目录
├── docs # 存放文档目录
├── example # 存放代码示例的目录
├── makefile # 项目的 makefile 文件
├── scripts # 存放项目需要的相关脚本文件的目录
├── src # 存放 Rust 相关代码的源文件的目录
├── target # 编译后生成的文件目录
└── tests # 存放项目测试用例相关文件的目录
```

## Src 目录

Src 目录主要关注：clients、cmd、common、journal-server、mqtt-bridge、mqtt-broker、placement-center、protocol、storage-adapter目录。

MQTT协议协议相关代码都在mqtt-bridge，Placement Center代码都在placement-center。其他的都是给这两个模块服务的。

代码入口在cmd模块中，有mqtt-server、placement-center、journal-server三个模块，分别是三个组件的入口。所以代码可以从这里开始看。

配置、log相关的通用代码在common/base中。

```
.
├── amqp-broker # AMAP 协议 Broker目录的源文件，暂没启用
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── amqp-plugins # AMAP 协议 Broker 插件目录的源文件，暂没启用
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── cli-command # RobustMQ 命令行工具cli的源文件，暂没启用
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── clients # Placement Center服务的客户端的目录
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── cmd # 项目启动的入口
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── common # 存放通用代码和模块的目录
│   ├── base # 存放通用代码，比如日志、配置，工具类等
│   ├── metadata-struct # 存放多个项目都会用到的结构体
│   ├── raft-rocksdb # 存放Raft 和RocksDB的通用代码，暂未启用
│   ├── rocksdb-engine # 存放RocksDB Engine通用代码，暂未启用
│   └── third-driver # 存放三方驱动类，比如MySQL
├── journal-remote # Journal数据存放到远程的源文件，暂没启用
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── journal-server # Journal Server 项目的代码源文件
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── mqtt-bridge # MQTT 桥接功能的代码源文件
│   ├── elasticsearch
│   ├── kafka
│   └── redis
├── mqtt-broker # MQTT broker 模块的项目源代码
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── placement-center # Placement Center 模块的项目源代码
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── protocol # RobustMQ 项目所有协议解析相关的源代码
│   ├── Cargo.toml
│   ├── src
│   └── tests
└── storage-adapter # Storage Adapter模块的项目源代码
    ├── Cargo.toml
    ├── src
    └── tests

```
