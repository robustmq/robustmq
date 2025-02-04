client -> server

四种通信模型
1. 简单RPC 客户端单一请求 服务器单一响应
2. 服务器流式RPC 客户端单一请求 服务端流式响应
3. 客户端流式RPC 客户端流式请求 服务端单一响应
4. 双向流式RPC 客户端流式请求 服务端流式请求

client 发送 请求信息 給 server
server 处理 请求信息 来自 client
server 发送 响应信息 给 client
client 处理 响应信息 来自 server



接口名称: RegisterNode

通信模式: 简单RPC

Client: Mqtt Broker

Server: Placement Center

请求信息:

// The type of the cluster.
enum ClusterType{
// The type of the PlacementCenter.
PlacementCenter = 0;
// The type of the JournalServer.
JournalServer = 1;
// The type of the MQTTBrokerServer.
MQTTBrokerServer = 2;
// The type of the AMQPBrokerServer.
AMQPBrokerServer = 3;
}

message RegisterNodeRequest{
ClusterType cluster_type = 1;
string cluster_name = 2;
string node_ip = 3;
uint64 node_id = 4;
string node_inner_addr = 5;
string extend_info = 6;
}

如何处理请求




响应信息

message RegisterNodeReply{

}

如何处理响应信息

不重要
