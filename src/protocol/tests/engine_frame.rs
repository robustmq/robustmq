#[cfg(test)]
mod tests {

    use futures::{SinkExt, StreamExt};
    use prost::Message;
    use protocol::storage_engine::{
        codec::{StorageEngineCodec, StorageEnginePacket},
        storage::{
            ApiKey, ApiType, ApiVersion, Header, ProduceReq, ProduceReqBody, ProduceResp,
            ProduceRespBody, RequestCommon,
        },
    };
    use tokio::net::{TcpListener, TcpStream};
    use tokio_util::codec::Framed;



    fn build_produce_req() -> StorageEnginePacket {
        let header = Header {
            api_key: ApiKey::Produce.into(),
            api_type: ApiType::Request.into(),
            api_version: ApiVersion::V0.into(),
            request: Some(RequestCommon {
                correlation_id: 3,
                client_id: "testsssss".to_string(),
            }),
            response: None,
        };

        let body = ProduceReqBody {
            transactional_id: 1,
            acks: 1,
            timeout_ms: 60000,
            topic_data: None,
        };
        let req = ProduceReq {
            header: Some(header),
            body: Some(body),
        };
        return StorageEnginePacket::ProduceReq(req);
    }

    fn build_produce_resp() -> StorageEnginePacket {
        let header = Header {
            api_key: ApiKey::Produce.into(),
            api_type: ApiType::Response.into(),
            api_version: ApiVersion::V0.into(),
            request: None,
            response: None,
        };

        let body = ProduceRespBody {};
        let req = ProduceResp {
            header: Some(header),
            body: Some(body),
        };
        return StorageEnginePacket::ProduceResp(req);
    }

    #[test]
    fn encode_size() {
        let header = Header {
            api_key: ApiKey::Produce.into(),
            api_type: ApiType::Request.into(),
            api_version: ApiVersion::V0.into(),
            request: Some(RequestCommon {
                correlation_id: 3,
                client_id: "testsssss".to_string(),
            }),
            response: None,
        };

        let req = ProduceReq {
            header: Some(header.clone()),
            body: None,
        };
        let body = ProduceReq::encode_to_vec(&req);
        println!("ProduceReq size:{}", body.len());
        println!("ProduceReq data:{:?}", ProduceReq::decode(body.as_slice()));

        let h_body = Header::encode_to_vec(&header);
        println!("Header size:{}", h_body.len());
        println!("Header data:{:?}", Header::decode(h_body.as_slice()));
    }
}
