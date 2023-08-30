<p  align="center">
  <picture>
    <img alt="GreptimeDB Logo" src="image/logo2.jpg" width="220" height="90">
  </picture>
</p>
 <h3 align="center">
    The next-generation Cloud native Message Queue with extremely high performance.
</h3>
<p align="center">
    <a href="https://codecov.io/gh/GrepTimeTeam/greptimedb"><img src="https://codecov.io/gh/GrepTimeTeam/greptimedb/branch/develop/graph/badge.svg?token=FITFDI3J3C"></img></a>
    &nbsp;
    <a href="https://github.com/GreptimeTeam/greptimedb/actions/workflows/develop.yml"><img src="https://github.com/GreptimeTeam/greptimedb/actions/workflows/develop.yml/badge.svg" alt="CI"></img></a>
    &nbsp;
    <a href="https://github.com/greptimeTeam/greptimedb/blob/develop/LICENSE"><img src="https://img.shields.io/github/license/greptimeTeam/greptimedb"></a>
</p>

<p align="center">
    <a href="https://twitter.com/greptime"><img src="https://img.shields.io/badge/twitter-follow_us-1d9bf0.svg"></a>
    &nbsp;
    <a href="https://www.linkedin.com/company/greptime/"><img src="https://img.shields.io/badge/linkedin-connect_with_us-0a66c2.svg"></a>
    &nbsp;
    <a href="https://greptime.com/slack"><img src="https://img.shields.io/badge/slack-GreptimeDB-0abd59?logo=slack" alt="slack" /></a>
</p>

## What is RobustMQ?
RobustMQ is a multi-protocol, cloud-native, Serverless, and simple architecture converged message queue with extremely high performance and modular plugin for different storages. 

- Multi-protocol support

It is hoped that its architecture can naturally support the easy adaptation of multiple MQ protocols. RobustMQ will not customize private protocols in the short term, and the goal is to adapt to a variety of mainstream protocols in the industry, so as to meet the different demands of existing users. To reduce the user's education costs and switching costs.

- Cloud-native /Serverless

It is hoped that it naturally supports containerized deployment, supports elastic Serverless computing layer and storage layer architecture, and can rapidly expand and shrink capacity. In order to solve the problem of cluster elasticity, improve the utilization rate of cluster, so as to realize the payment according to quantity and use on demand.

- Extremely High performance

it is developed by Rust programming language which is well known as its outstanding performance and smallest memory. RobustMQ tends to be the best performer in Message Queue industry.

- Plugin modular for different storages

it is designed with flexible architecture with different plugins to support different storages such as local storage, remote storage and classified storages (Local for hot data and remote for cold data). So that it can fully tap into the cost of different storages and makes RobustMQ the most economical product.

- The architecture is simple 

it is expected that its system architecture is only composed of brokers, and the cluster is formed by AD hoc networking between brokers. That is, at least one of them can be used to build a cluster, and it also has the ability to expand the cluster horizontally. To reduce deployment and O&M costs. So that it can meet the edge computing scenarios and cloud computing central cluster scenarios.

## Architecture
![架构图](image/robustmq-architecture.png)
