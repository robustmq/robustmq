## System Architecture

![image](../../images/doc-image5.png)

As shown in the figure above: RobustMQ MQTT consists of three parts: MQTT Broker, Placement Center, and Storage Engine.

The Placement Center is the metadata management center for the RobustMQ MQTT cluster and is responsible for metadata management of the MQTT cluster, node management of the cluster, failure recovery of the cluster, and more.

MQTT brokers are completely stateless nodes, and MQTT clients randomly access the Pub/Sub where a Broker completes the message data. MQTT Broker completes node discovery and node exploration based on Placement Center, thus completing cluster construction.

The MQTT cluster persistently stores message data to the Storage Engine through the Storage Adapter layer.

The metadata of the MQTT Cluster is stored in the Placement Center Cluster. MQTT Broker supports the parsing of MQTT 3/4/5 protocol based on TCP and the internal control and scheduling of the cluster based on GRPC protocol.

The Placement Center will run the controller thread corresponding to the MQTT Broker cluster. Responsible for scheduling MQTT clusters, such as the Leader of the shared cluster.