#[cfg(test)]
mod tests {
    use common::runtime::create_runtime;
    use protocol::placement_center::placement::{placement_center_service_client::PlacementCenterServiceClient, RegisterNodeRequest};
    use tokio::runtime::Runtime;

    #[test]
    fn set() {
        let runtime: Runtime = create_runtime("meta-test", 3);

        let _gurad = runtime.enter();

        runtime.block_on(async move {
            let mut client = PlacementCenterServiceClient::connect("http://127.0.0.1:1228")
                .await
                .unwrap();

            let request = tonic::Request::new(RegisterNodeRequest::default());
            let response = client.register_node(request).await.unwrap();

            println!("response={:?}", response);
        });
    }
}
