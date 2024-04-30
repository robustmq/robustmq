#[cfg(test)]
mod tests {
    use protocol::placement_center::generate::{
        common::ClusterType,
        journal::{
            engine_service_client::EngineServiceClient, CreateSegmentRequest, CreateShardRequest,
            DeleteSegmentRequest, DeleteShardRequest,
        },
        placement::{
            placement_center_service_client::PlacementCenterServiceClient, HeartbeatRequest,
            RegisterNodeRequest, UnRegisterNodeRequest,
        },
    };

    #[tokio::test]
    async fn test_register_node() {
        let mut client = PlacementCenterServiceClient::connect("http://127.0.0.1:1228")
            .await
            .unwrap();

        let mut request = RegisterNodeRequest::default();
        request.cluster_type = cluster_type();
        request.cluster_name = cluster_name();
        request.node_id = node_id();
        request.node_ip = node_ip();
        request.extend_info = extend_info();
        let response = client
            .register_node(tonic::Request::new(request))
            .await
            .unwrap();

        println!("response={:?}", response);
    }

    #[tokio::test]
    async fn test_heartbeat() {
        let mut client = PlacementCenterServiceClient::connect("http://127.0.0.1:1228")
            .await
            .unwrap();

        let mut request = HeartbeatRequest::default();
        request.cluster_type = cluster_type();
        request.cluster_name = cluster_name();
        request.node_id = node_id();
        let response = client
            .heartbeat(tonic::Request::new(request))
            .await
            .unwrap();

        println!("response={:?}", response);
    }

    #[tokio::test]
    async fn test_unregister_node() {
        let mut client = PlacementCenterServiceClient::connect("http://127.0.0.1:1228")
            .await
            .unwrap();

        let mut request = UnRegisterNodeRequest::default();
        request.cluster_type = cluster_type();
        request.cluster_name = cluster_name();
        request.node_id = node_id();
        let response = client
            .un_register_node(tonic::Request::new(request))
            .await
            .unwrap();

        println!("response={:?}", response);
    }

    #[tokio::test]
    async fn test_create_shard() {
        let mut client = EngineServiceClient::connect("http://127.0.0.1:1228")
            .await
            .unwrap();

        let mut request = CreateShardRequest::default();
        request.cluster_name = cluster_name();
        request.shard_name = shard_name();
        request.replica = shard_replica();
        let response = client
            .create_shard(tonic::Request::new(request))
            .await
            .unwrap();

        println!("response={:?}", response);
    }

    #[tokio::test]
    async fn test_delete_shard() {
        let mut client = EngineServiceClient::connect("http://127.0.0.1:1228")
            .await
            .unwrap();

        let mut request = DeleteShardRequest::default();
        request.cluster_name = cluster_name();
        request.shard_name = shard_name();
        let response = client
            .delete_shard(tonic::Request::new(request))
            .await
            .unwrap();

        println!("response={:?}", response);
    }

    #[tokio::test]
    async fn test_create_segment() {
        let mut client = EngineServiceClient::connect("http://127.0.0.1:1228")
            .await
            .unwrap();

        let mut request = CreateSegmentRequest::default();
        request.cluster_name = cluster_name();
        request.shard_name = shard_name();
        let response = client
            .create_segment(tonic::Request::new(request))
            .await
            .unwrap();

        println!("response={:?}", response);
    }

    #[tokio::test]
    async fn test_delete_segment() {
        let mut client = EngineServiceClient::connect("http://127.0.0.1:1228")
            .await
            .unwrap();

        let mut request = DeleteSegmentRequest::default();
        request.cluster_name = cluster_name();
        request.shard_name = shard_name();
        request.segment_seq = 1;
        let response = client
            .delete_segment(tonic::Request::new(request))
            .await
            .unwrap();

        println!("response={:?}", response);
    }

    fn shard_name() -> String {
        return "test1".to_string();
    }

    fn shard_replica() -> u32 {
        return 1;
    }

    fn cluster_type() -> i32 {
        return ClusterType::StorageEngine.into();
    }
    fn cluster_name() -> String {
        return "tokio-test2".to_string();
    }

    fn node_id() -> u64 {
        return 4;
    }

    fn node_ip() -> String {
        return "127.0.0.4".to_string();
    }

    fn node_port() -> u32 {
        return 2287;
    }

    fn extend_info() -> String {
        return "extend info".to_string();
    }
}
