# Code Structure

## Root Directory

Focus mainly on the following files and directories: `bin`, `config`, `docs`, `makefile`, `scripts`, `src`, `tests`.

```
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── README.md
├── benches  # Directory for benchmarking-related files
├── bin # Directory for service startup scripts
├── build.rs # Cargo's build.rs file
├── config # Directory for project configurations
├── docs # Directory for documentation
├── example # Directory for code examples
├── makefile # The project's makefile
├── scripts # Directory for related scripts needed by the project
├── src # Directory for Rust source files
├── target # Directory for compiled files
└── tests # Directory for project test cases
```

## Src Directory

The `src` directory primarily focuses on the following directories: `clients`, `cmd`, `common`, `journal-server`, `mqtt-bridge`, `mqtt-broker`, `placement-center`, `protocol`, `storage-adapter`.

The code related to the MQTT protocol is in `mqtt-bridge`, and the code for Placement Center is in `placement-center`. The other directories serve these two modules.

The entry point for the code is in the `cmd` module, which includes `mqtt-server`, `placement-center`, and `journal-server` modules. These are the entry points for the three components, so you can start reading the code from here.

Common code related to configuration and logging is in `common/base`.

```
.
├── amqp-broker # Source files for the AMQP protocol Broker directory, not currently in use
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── amqp-plugins # Source files for the AMQP protocol Broker plugins, not currently in use
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── cli-command # Source files for the RobustMQ command-line tool cli, not currently in use
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── clients # Directory for clients of the Placement Center service
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── cmd # Entry point for project startup
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── common # Directory for common code and modules
│   ├── base # Common code, such as logging, configuration, utility classes, etc.
│   ├── metadata-struct # Structures used by multiple projects
│   ├── raft-rocksdb # Common code for Raft and RocksDB, not currently in use
│   ├── rocksdb-engine # Common code for RocksDB Engine, not currently in use
│   └── third-driver # Third-party driver classes, such as MySQL
├── journal-remote # Source files for storing Journal data remotely, not currently in use
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── journal-server # Source files for the Journal Server project
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── mqtt-bridge # Source files for MQTT bridging functionality
│   ├── elasticsearch
│   ├── kafka
│   └── redis
├── mqtt-broker # Source code for the MQTT broker module
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── placement-center # Source code for the Placement Center module
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── protocol # Source code for all protocol parsing in the RobustMQ project
│   ├── Cargo.toml
│   ├── src
│   └── tests
└── storage-adapter # Source code for the Storage Adapter module
    ├── Cargo.toml
    ├── src
    └── tests
```
