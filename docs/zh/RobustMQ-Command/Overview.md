# 概述

## 整体介绍

RobustMQ Command 是 RobustMQ 提供的命令行工具，用于集群相关操作。
目前包含 mqtt、place、journal 三个模块，分别对应 RobustMQ 的 MQTT Broker、
元数据服务 Placement Center、存储层 Journal Server 三个组件。

> [!NOTE]
> 当前版本的 RobustMQ Command 仅支持单机数据的获取，多集群数据获取还在支持中, 欢迎
> 帮忙构建。

```
$ ./bin/robust-ctl
Command line tool for RobustMQ

Usage: robust-ctl <COMMAND>

Commands:
  mqtt
          Command line tool for mqtt broker
  place
          Command line tool for placement center
  journal
          Command line tool for journal engine
  help
          Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help
  -V, --version
          Print version
```

## 特别注意

`robust-ctl` 命令是基于最终构建产物进行执行的，因此在执行上述命令前需要先进行构建。

```bash
# 构建
make build

# 解压压缩包
tar -zxvf build/robustmq-xxx.tar.gz

# 使用
./build/robustmq-xxx/bin/robust-ctl
```

此处仅作演示说明，后续命令都使用 `./bin/robust-ctl` 代指当前命令。

### 使用 cargo 执行

本地开发时，可以简化该流程使用 cargo 执行。

```bash
cargo run --package cmd --bin cli-command
```

如果需要指定参数则如下执行即可：

```bash
cargo run --package cmd --bin cli-command -- arg1 arg2 ...
```

此处仅作演示说明，后续命令都使用 `./bin/robust-ctl` 代指当前命令。

## MQTT Broker

负责 MQTT Broker 服务相关的操作

```
$ ./bin/robust-ctl mqtt -h
Command line tool for mqtt broker

Usage: robust-ctl mqtt [OPTIONS] <COMMAND>

Commands:
  status

  user
          related operations of mqtt users, such as listing, creating, and deleting
  list-connection

  list-topic
          action: list topics
  publish
          Command line tool for mqtt broker
  subscribe
          Command line tool for mqtt broker
  slow-sub

  help
          Print this message or the help of the given subcommand(s)

Options:
  -s, --server <SERVER>
          [default: 127.0.0.1:9981]
  -h, --help
          Print help
```

## Placement Center

负责 Placement Center 服务相关的操作

```
$ ./bin/robust-ctl place -h
Command line tool for placement center

Usage: robust-ctl place [OPTIONS] <COMMAND>

Commands:
  status

  add-learner
          action: add learner
  change-membership
          action: change membership
  help
          Print this message or the help of the given subcommand(s)

Options:
  -s, --server <SERVER>
          [default: 127.0.0.1:1228]
  -h, --help
          Print help
```

## Journal Server

负责 Journal Server 服务相关的操作

```
$ ./bin/robust-ctl journal -h
Command line tool for journal engine

Usage: robust-ctl journal [OPTIONS]

Options:
  -s, --server <SERVER>
          [default: 127.0.0.1:1228]
  -a, --action <ACTION>
          [default: status]
  -h, --help
          Print help
```
