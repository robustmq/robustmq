#[cfg(test)]
mod tests {
    use common::runtime::create_runtime;
    use protocol::robust::meta::{meta_service_client::MetaServiceClient, BrokerUnRegisterRequest};
    use tokio::runtime::Runtime;

    #[test]
    fn broker_un_register() {
        let runtime: Runtime = create_runtime("meta-test", 3);

        let _gurad = runtime.enter();

        runtime.block_on(async move {
            let mut client = MetaServiceClient::connect("http://127.0.0.1:1228")
                .await
                .unwrap();

            let request = tonic::Request::new(BrokerUnRegisterRequest { node_id: 1 });
            let response = client.broker_un_register(request).await.unwrap();

            println!("response={:?}", response);
        });
    }
}
