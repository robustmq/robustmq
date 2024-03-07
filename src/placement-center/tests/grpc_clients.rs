#[cfg(test)]
mod tests {
    use protocol::placement_center::placement::{
        placement_center_service_client::PlacementCenterServiceClient, ClusterType, RegisterNodeRequest
    };

    #[tokio::test]
    async fn register_node() {
        let mut client = PlacementCenterServiceClient::connect("http://127.0.0.1:1228")
            .await
            .unwrap();

        let mut request = RegisterNodeRequest::default();
        request.cluster_type = ClusterType::StorageEngine.into();
        request.cluster_name = "tokio-test".to_string();
        request.node_id = 2;
        request.node_ip = "127.0.0.1".to_string();
        request.node_port = 2287;
        request.extend_info = "extend info".to_string();
        let response = client.register_node(tonic::Request::new(request)).await.unwrap();

        println!("response={:?}", response);
    }
}
