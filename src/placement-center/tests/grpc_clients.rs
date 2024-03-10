#[cfg(test)]
mod tests {
    use protocol::placement_center::placement::{
        placement_center_service_client::PlacementCenterServiceClient, ClusterType,
        HeartbeatRequest, RegisterNodeRequest, UnRegisterNodeRequest,
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
        request.node_port = node_port();
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

    fn cluster_type() -> i32 {
        return ClusterType::StorageEngine.into();
    }
    fn cluster_name() -> String {
        return "tokio-test2".to_string();
    }

    fn node_id() -> u64 {
        return 3;
    }

    fn node_ip() -> String {
        return "127.0.0.3".to_string();
    }

    fn node_port() -> u32 {
        return 2287;
    }

    fn extend_info() -> String {
        return "extend info".to_string();
    }
}
