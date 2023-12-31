<p  align="center">
  <picture>
    <img alt="RobustMQ Logo" src="docs/RobustMQ-logo-formal.png" width="300">
  </picture>
</p>
 <h3 align="center">
    The next-generation Cloud native Message Queue with extremely high performance.
</h3>

## What is RobustMQ?
RobustMQ is a multi-protocol, cloud-native, Serverless, and simple architecture converged message queue with extremely high performance and modular plugin for different storages. 

- Multi-protocol support

It is hoped that its architecture can naturally support the easy adaptation of multiple MQ protocols. RobustMQ will not customize private protocols in the short term, and the goal is to adapt to a variety of mainstream protocols in the industry, so as to meet the different demands of existing users. To reduce the user's education costs and switching costs.

- Cloud-native /Serverless

It is hoped that it naturally supports containerized deployment, supports elastic Serverless computing layer and storage layer architecture, and can rapidly expand and shrink capacity. In order to solve the problem of cluster elasticity, improve the utilization rate of cluster, so as to realize the payment according to quantity and use on demand.

- Extremely high performance

It is developed by Rust programming language which is well known as its outstanding performance and smallest memory. RobustMQ tends to be the best performer in Message Queue industry.

- Plugin modular for different storages

It is designed with flexible architecture with different plugins to support different storages such as local storage, remote storage and classified storages (Local for hot data and remote for cold data). So that it can fully tap into the cost of different storages and makes RobustMQ the most economical product.

- The architecture is simple 

It is expected that its system architecture is only composed of brokers, and the cluster is formed by AD hoc networking between brokers. That is, at least one of them can be used to build a cluster, and it also has the ability to expand the cluster horizontally. To reduce deployment and O&M costs. So that it can meet the edge computing scenarios and cloud computing central cluster scenarios.

### Get Start
- Cargo run starts the service and displays help information:
```
# cargo run -- --help
RobustMQ: Next generation cloud-native converged high-performance message queue.

Usage: robustmq-server [OPTIONS]

Options:
  -s, --server-conf <SERVER_CONF>
          broker server configuration file path [default: config/server.toml]
  -m, --meta-conf <META_CONF>
          MetaService Indicates the path of the configuration file [default: config/meta.toml]
  -h, --help
          Print help
  -V, --version
```
- Cargo run starts the service:
```
cargo run
```
- Cargo test runs the test cases in libraries
```
cargo test -p lib_name(e.g. protocol) --lib
```

## Architecture
![Architecture Image](docs/robustmq-architecture.png)

## License
RobustMQ uses the Apache 2.0 license to strike a balance between open contributions and allowing you to use the software however you want.

## Contributing
Please refer to contribution [guidelines](https://github.com/robustmq/robustmq) for more information.

