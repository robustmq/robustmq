use protocol::journal_server::{
    codec::StorageEnginePacket,
    generate::protocol::{
        header::{ApiKey, ApiType, ApiVersion, Header, RequestCommon, ResponseCommon},
        produce::{ProduceResp, ProduceRespBody},
    },
};

pub fn build_produce_resp() -> StorageEnginePacket {
    let header = build_req_header(33, "client-id-fsfsdff".to_string());
    let body = ProduceRespBody {};
    let req = ProduceResp {
        header: Some(header),
        body: Some(body),
    };
    return StorageEnginePacket::ProduceResp(req);
}

fn build_req_header(correlation_id: u32, client_id: String) -> Header {
    return Header {
        api_key: ApiKey::Produce.into(),
        api_type: ApiType::Response.into(),
        api_version: ApiVersion::V0.into(),
        request: Some(RequestCommon {
            correlation_id: correlation_id,
            client_id: client_id,
        }),
        response: None,
    };
}
