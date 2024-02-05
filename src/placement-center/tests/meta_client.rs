#[cfg(test)]
mod tests {
    use common::runtime::create_runtime;
    use protocol::robust::meta::{meta_service_client::MetaServiceClient, SetRequest};
    use tokio::runtime::Runtime;

    #[test]
    fn set() {
        let runtime: Runtime = create_runtime("meta-test", 3);

        let _gurad = runtime.enter();

        runtime.block_on(async move {
            let mut client = MetaServiceClient::connect("http://127.0.0.1:1228")
                .await
                .unwrap();

            let request = tonic::Request::new(SetRequest::default());
            let response = client.set(request).await.unwrap();

            println!("response={:?}", response);
        });
    }
}
