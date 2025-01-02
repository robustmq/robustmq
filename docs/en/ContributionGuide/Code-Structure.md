# Code Structure
## Root Directory

Mainly focus on the following files and directories: bin, config, docs, makefile, scripts, src, and tests.
```
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── README.md
├── benches  # Directory for storing files related to load testing
├── bin      # Directory for storing script files to start services
├── build.rs # The build.rs file of Cargo
├── config   # Directory for storing project configurations
├── docs     # Directory for storing documentation
├── example  # Directory for storing code examples
├── makefile # The makefile of the project
├── scripts  # Directory for storing relevant script files required by the project
├── src      # Directory for storing source files of Rust - related code
├── target   # Directory for files generated after compilation
└── tests    # Directory for storing files related to project test cases
```

## Src Directory

In the Src directory, mainly focus on the following directories: clients, cmd, common, journal - server, mqtt - bridge, mqtt - broker, placement - center, protocol, and storage - adapter.

The code related to the MQTT protocol is in the mqtt - bridge, and the code for the Placement Center is in the placement - center. The rest are for serving these two modules.

The code entry points are in the cmd module, which has three sub - modules: mqtt - server, placement - center, and journal - server, which are the entry points for the three components respectively. So, you can start looking at the code from here.

The common code related to configuration and logging is in common/base.
```
.
├── amqp - broker # Source files of the AMAP protocol Broker directory, not enabled yet
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── amqp - plugins # Source files of the AMAP protocol Broker plugin directory, not enabled yet
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── cli - command # Source files of the RobustMQ command - line tool cli, not enabled yet
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── clients # Directory for clients of the Placement Center service
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── cmd # Entry point for starting the project
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── common # Directory for storing common code and modules
│   ├── base # Storing common code, such as logging, configuration, utility classes, etc.
│   ├── metadata - struct # Storing structs used in multiple projects
│   ├── raft - rocksdb # Storing common code for Raft and RocksDB, not enabled yet
│   ├── rocksdb - engine # Storing common code for RocksDB Engine, not enabled yet
│   └── third - driver # Storing third - party drivers, such as MySQL
├── journal - remote # Source files for storing Journal data remotely, not enabled yet
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── journal - server # Source files of the Journal Server project
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── mqtt - bridge # Source files of the MQTT bridging functionality
│   ├── elasticsearch
│   ├── kafka
│   └── redis
├── mqtt - broker # Source code of the MQTT broker module project
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── placement - center # Source code of the Placement Center module project
│   ├── Cargo.toml
│   ├── src
│   └── tests
├── protocol # Source code related to all protocol parsing in the RobustMQ project
│   ├── Cargo.toml
│   ├── src
│   └── tests
└── storage - adapter # Source code of the Storage Adapter module project
    ├── Cargo.toml
    ├── src
    └── tests
```
