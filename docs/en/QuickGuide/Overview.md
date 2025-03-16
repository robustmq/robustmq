## Overview
RobustMQ is expected to support multiple message queuing protocols in the long term. The MQTT protocol is currently supported, called RobustMQ MQTT.

## RobustMQ MQTT

### Deployment patterns
RobustMQ MQTT has two deployment modes: stand-alone and cluster.
- Stand-alone mode: Start the MQTT Server in stand-alone mode, where both the Placement Center and the MQTT Server are running as stand-alone.
- Cluster mode: Start MQTT Server in cluster mode, where the Placement Center and MQTT Server are both running in multi-node cluster mode. The Placement Center has three nodes by default, and the number of MQTT Broker nodes is not limited.

## Mode of operation
1. Cargo run: Download the source code and run the MQTT Server with the cargo run command. This method is suitable for development and debugging.
2. Binary package run: Download or compile the binary package, then execute the binary package to run the MQTT Server. This approach is suitable for production environments.
3. Docker run: that is, download or compile the Docker image, then execute the Docker image to run the MQTT Server. This approach is suitable for production environments.
4. K8s run: That is, run MQTT Server in the K8s cluster. This approach is suitable for production environments.
   
> Recommendation: During the development and debugging phase, we generally use Cargo to run. In production, we generally recommend running with docker or K8s because of the ease of scaling up and down. We also support binary packages.

### Quick start
- [Compiling binary packages [optional]](mqtt/Build.md)
- [Binary run - single machine mode](mqtt/Run-Standalone-Mode.md)
- [Binary run-cluster mode](mqtt/Run-Cluster-Mode.md)
- [Docker Run](mqtt/Run-Docker-Mode.md)
- [K8s Run](mqtt/Run-K8S-Mode.md)