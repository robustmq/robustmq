## Project Origin
Write the first line of code in October 2023. There were two goals:
1. Learn Rust. Rust is a language with a high barrier to entry. It needs to be learned and mastered in the real world. I couldn't find a suitable project at that time, so I decided to write my own.
2. Explore Rust's use of message queues. Any open source component in the message queue space has issues. I want to start a new wheel and try to see if I can solve these problems without the burden of history.

## About License
Apache License 2.0, our goal is to enter the Apache Foundation. Super cool.

## Now the message queue component is doing something
Some of the things that mainstream message queuing components are currently doing:
-Separation of storage and computation: solving the problem of elastic expansion and shrinkage of the system.
- Multi-protocol (fusion of message and stream) : to solve the problem that multiple scenarios need to adapt to multiple protocols, and the cost of user switching protocol client transformation is high.
- Remove metadata dependency: Current mainstream message queue architectures rely on metadata components such as Zookeeper, but due to the complexity and stability of metadata components themselves, it also leads to stability and availability problems of message queues.
- Hierarchical storage: You want to reduce the cost of storage by moving some cold data to a lower-cost storage like S3, HDFS, etc.
- Adapt to different storage engines: Depending on the use case, you may need to adapt to different storage engines, such as S3, HDFS, in-house storage engines, etc.

In doing this, we found two types of problems:
- Native architecture defects lead to high complexity of the transformation, requiring a lot of human and material resources, and low revenue.
- It is not compatible with the community after the transformation, it cannot follow the evolution of the community, and it cannot be maintained in the long run.
- Each company will have its own version of the transformation, unable to form unity and synergy, repeated investment.
-

## What do we wish we were
- is a pure message queuing component that doesn't bother to do anything around it until it's done with the core message queuing functionality.
- Is a platform-wide message queue that can be adapted to all popular message queuing protocols to support a variety of scenarios.
- Is a fully Serverless message queue, fully resilient, capable of unperceived horizontal scaling and horizontal scaling.
- It is a pluggable message queue that can support different storage requirements in different scenarios. It can be configured to switch between different storage engines, such as S3, HDFS, Bookeeper, etc.
- Is a high performance, stable, reliable message queue, able to support a large number of connections, QPS, throughput.


## And existing message queues
Not yet dare to say the difference, salute the community. But as we continue to try, we will be able to make a message queue product that is better than the current open source components.

## Will we be competitive in the long run
There must be some good, absolutely confident.

Combined with the current development of Rust, cloud native, AI and other fields, continuous integration and learning, on the basis of no historical baggage. We are absolutely confident that we will be able to build a competitive message queue product for the AI age.

The key words are: multi-protocol, Serverless, plug-in storage. That is:

>  Based on Rust language, create a message queue product that supports multi-protocol, supports complete Serverless capabilities in the architecture, and has plug-in storage capabilities.

This definition will be refined over time, and the combination of MQ and AI will be explored next. And there are some explorations in the financial level Message Queue.


## Expectation
I hope we stand on the shoulders of giants, look up at the sky, walk with heads down, work at ease, and build the best MQ Infra component.
