#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use protocol::robust::meta::{meta_service_client::MetaServiceClient, FindLeaderRequest};
    use tokio::runtime::Runtime;

    #[test]
    fn grpc_client() {
        let runtime: Runtime = tokio::runtime::Builder::new_multi_thread()
            // .worker_threads(self.config.work_thread.unwrap() as usize)
            .max_blocking_threads(2048)
            .thread_name("meta-http")
            .enable_io()
            .build()
            .unwrap();

        let _gurad = runtime.enter();

        runtime.spawn(async move {
            let mut client = MetaServiceClient::connect("http://127.0.0.1:1228")
                .await
                .unwrap();

            let request = tonic::Request::new(FindLeaderRequest {});
            let response = client.find_leader(request).await.unwrap();

            println!("response={:?}", response);
        });

        loop {
            thread::sleep(Duration::from_secs(300));
        }
    }
}
