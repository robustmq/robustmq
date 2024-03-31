#[cfg(test)]
mod tests {
    use protocol::placement_center::generate::kv::{
        kv_service_client::KvServiceClient, DeleteRequest, ExistsRequest, GetRequest, SetRequest,
    };

    #[tokio::test]
    async fn kv_storage() {
        let mut client = KvServiceClient::connect("http://127.0.0.1:1228")
            .await
            .unwrap();
        let key = "test".to_string();
        let value = "test_value".to_string();
        let set_req = SetRequest {
            key: key.clone(),
            value: value.clone(),
        };
        let _ = client.set(set_req).await;

        let get_req = GetRequest { key: key.clone() };
        let get_rep = client.get(get_req).await.unwrap().into_inner();
        assert_eq!(value, get_rep.value);

        let exists_req = ExistsRequest { key: key.clone() };
        let ex_rep = client.exists(exists_req).await.unwrap().into_inner();
        assert!(ex_rep.flag);

        let del_req = DeleteRequest { key: key.clone() };
        let _ = client.delete(del_req).await.unwrap().into_inner();

        let exists_req = ExistsRequest { key: key.clone() };
        let ex_rep = client.exists(exists_req).await.unwrap().into_inner();
        assert!(!ex_rep.flag);
    }
}
