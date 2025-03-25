# Overview

## Commands

RobustMQ Command is a command-line tool from RobustMQ for cluster-related operations. Currently, it contains three modules, mqtt, place, and journal, which correspond to the three components of the RobustMQ: MQTT Broker, the metadata service Placement Center, and the storage layer Journal Server.

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

## MQTT Broker

Responsible for MQTT Broker service related operations

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

Responsible for the Placement Center service related operations

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

Responsible for Journal Server service related operations

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
