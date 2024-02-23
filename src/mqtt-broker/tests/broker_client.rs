#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};
    use common::runtime::create_runtime;
    use protocol::broker_server::broker::{broker_service_client::BrokerServiceClient, StopBrokerRequest};

    #[test]
    fn stop_broker() {
        let rt = create_runtime("broker-client", 3);
        rt.block_on(async {
            let mut client = BrokerServiceClient::connect("http://127.0.0.1:9987")
                .await
                .unwrap();

            let request = tonic::Request::new(StopBrokerRequest {});
            let response = client.stop_broker(request).await.unwrap();

            println!("response={:?}", response);

            loop {
                thread::sleep(Duration::from_secs(300));
            }
        });
    }
}
