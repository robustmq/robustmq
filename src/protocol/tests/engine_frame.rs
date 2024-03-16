#[cfg(test)]
mod tests {
    use std::io::Read;

    use futures::{SinkExt, StreamExt};
    use prost::Message;
    use protocol::storage_engine::{
        codec::{StorageEngineCodec, StorageEnginePacket},
        storage::{
            ApiKey, ApiType, ApiVersion, Header, ProduceReq, ProduceReqBody, ProduceResp,
            ProduceRespBody,
        },
    };
    use tokio::net::{TcpListener, TcpStream};
    use tokio_util::codec::Framed;
    use tonic::IntoRequest;

    #[tokio::test]
    async fn storage_engine_frame_server() {
        let ip = "127.0.0.1:1228";
        let listener = TcpListener::bind(ip).await.unwrap();
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let mut stream = Framed::new(stream, StorageEngineCodec::new());
            tokio::spawn(async move {
                while let Some(Ok(data)) = stream.next().await {
                    println!("Got: {:?}", data);

                    // 发送的消息也只需要发送消息主体，不需要提供长度
                    // Framed/LengthDelimitedCodec 会自动计算并添加
                    //    let response = &data[0..5];
                    stream.send(build_produce_resp()).await.unwrap();
                }
            });
        }
    }

    #[tokio::test]
    async fn storage_engine_frame_client() {
        let socket = TcpStream::connect("127.0.0.1:1228").await.unwrap();
        let mut stream: Framed<TcpStream, StorageEngineCodec> =
            Framed::new(socket, StorageEngineCodec::new());

        // send connect package
        let _ = stream.send(build_produce_req()).await;

        let data = stream.next().await;
        println!("Got: {:?}", data);
    }

    fn build_produce_req() -> StorageEnginePacket {
        let header = Header {
            api_key: ApiKey::Produce.into(),
            api_type: ApiType::Request.into(),
            api_version: ApiVersion::V0.into(),
            request: None,
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
            request: None,
            response: None,
        };

        let da = Header::encode_to_vec(&header);
        let body = da.as_slice();
        println!("data = {:?}", Header::decode(body));
    }
}
