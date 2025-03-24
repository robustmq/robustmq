// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;

use protocol::journal_server::codec::JournalEnginePacket;
use protocol::journal_server::journal_engine::{
    ApiKey, ApiVersion, CreateShardReq, CreateShardReqBody, CreateShardRespBody, DeleteShardReq,
    DeleteShardReqBody, DeleteShardRespBody, FetchOffsetReq, FetchOffsetReqBody,
    FetchOffsetRespBody, GetClusterMetadataReq, GetClusterMetadataRespBody, GetShardMetadataReq,
    GetShardMetadataReqBody, GetShardMetadataReqShard, GetShardMetadataRespBody, ListShardReq,
    ListShardReqBody, ListShardRespBody, ReadReq, ReadReqBody, ReadRespBody, ReqHeader, WriteReq,
    WriteReqBody, WriteRespBody,
};

use crate::connection::ConnectionManager;
use crate::error::JournalClientError;
use crate::tool::resp_header_error;

pub(crate) async fn get_cluster_metadata(
    connection_manager: &Arc<ConnectionManager>,
) -> Result<GetClusterMetadataRespBody, JournalClientError> {
    let req_packet = JournalEnginePacket::GetClusterMetadataReq(GetClusterMetadataReq {
        header: Some(ReqHeader {
            api_key: ApiKey::GetClusterMetadata.into(),
            api_version: ApiVersion::V0.into(),
        }),
    });

    let resp_packet = connection_manager.admin_send(req_packet.clone()).await?;

    if let JournalEnginePacket::GetClusterMetadataResp(data) = resp_packet.clone() {
        resp_header_error(&data.header, resp_packet.clone())?;

        if let Some(body) = data.body {
            return Ok(body);
        }

        return Err(JournalClientError::ReceivedPacketNotContainBody(
            resp_packet.to_string(),
        ));
    }

    Err(JournalClientError::ReceivedPacketTypeError(
        req_packet.to_string(),
        resp_packet.to_string(),
    ))
}

pub(crate) async fn get_shard_metadata(
    connection_manager: &Arc<ConnectionManager>,
    shards: Vec<GetShardMetadataReqShard>,
) -> Result<GetShardMetadataRespBody, JournalClientError> {
    let req_packet = JournalEnginePacket::GetShardMetadataReq(GetShardMetadataReq {
        header: Some(ReqHeader {
            api_key: ApiKey::GetShardMetadata.into(),
            api_version: ApiVersion::V0.into(),
        }),
        body: Some(GetShardMetadataReqBody { shards }),
    });

    let resp_packet = connection_manager.admin_send(req_packet.clone()).await?;

    if let JournalEnginePacket::GetShardMetadataResp(data) = resp_packet {
        resp_header_error(&data.header, req_packet.clone())?;
        if let Some(body) = data.body {
            return Ok(body);
        }
        return Err(JournalClientError::ReceivedPacketNotContainBody(
            req_packet.to_string(),
        ));
    }

    Err(JournalClientError::ReceivedPacketTypeError(
        req_packet.to_string(),
        resp_packet.to_string(),
    ))
}

pub(crate) async fn create_shard(
    connection_manager: &Arc<ConnectionManager>,
    shard: CreateShardReqBody,
) -> Result<CreateShardRespBody, JournalClientError> {
    let req_packet = JournalEnginePacket::CreateShardReq(CreateShardReq {
        header: Some(ReqHeader {
            api_key: ApiKey::CreateShard.into(),
            api_version: ApiVersion::V0.into(),
        }),
        body: Some(shard),
    });

    let resp_packet = connection_manager.admin_send(req_packet.clone()).await?;

    if let JournalEnginePacket::CreateShardResp(data) = resp_packet {
        resp_header_error(&data.header, req_packet.clone())?;
        if let Some(body) = data.body {
            return Ok(body);
        }
        return Err(JournalClientError::ReceivedPacketNotContainBody(
            req_packet.to_string(),
        ));
    }

    Err(JournalClientError::ReceivedPacketTypeError(
        req_packet.to_string(),
        resp_packet.to_string(),
    ))
}

pub(crate) async fn delete_shard(
    connection_manager: &Arc<ConnectionManager>,
    shard: DeleteShardReqBody,
) -> Result<DeleteShardRespBody, JournalClientError> {
    let req_packet = JournalEnginePacket::DeleteShardReq(DeleteShardReq {
        header: Some(ReqHeader {
            api_key: ApiKey::DeleteShard.into(),
            api_version: ApiVersion::V0.into(),
        }),
        body: Some(shard),
    });

    let resp_packet = connection_manager.admin_send(req_packet.clone()).await?;

    if let JournalEnginePacket::DeleteShardResp(data) = resp_packet {
        resp_header_error(&data.header, req_packet.clone())?;
        if let Some(body) = data.body {
            return Ok(body);
        }
        return Err(JournalClientError::ReceivedPacketNotContainBody(
            req_packet.to_string(),
        ));
    }

    Err(JournalClientError::ReceivedPacketTypeError(
        req_packet.to_string(),
        resp_packet.to_string(),
    ))
}

pub(crate) async fn list_shard(
    connection_manager: &Arc<ConnectionManager>,
    shard: ListShardReqBody,
) -> Result<ListShardRespBody, JournalClientError> {
    let req_packet = JournalEnginePacket::ListShardReq(ListShardReq {
        header: Some(ReqHeader {
            api_key: ApiKey::ListShard.into(),
            api_version: ApiVersion::V0.into(),
        }),
        body: Some(shard),
    });

    let resp_packet = connection_manager.admin_send(req_packet.clone()).await?;

    if let JournalEnginePacket::ListShardResp(data) = resp_packet {
        resp_header_error(&data.header, req_packet.clone())?;
        if let Some(body) = data.body {
            return Ok(body);
        }
        return Err(JournalClientError::ReceivedPacketNotContainBody(
            req_packet.to_string(),
        ));
    }

    Err(JournalClientError::ReceivedPacketTypeError(
        req_packet.to_string(),
        resp_packet.to_string(),
    ))
}

pub(crate) async fn batch_write(
    connection_manager: &Arc<ConnectionManager>,
    node_id: u64,
    body: WriteReqBody,
) -> Result<WriteRespBody, JournalClientError> {
    let req_packet = JournalEnginePacket::WriteReq(WriteReq {
        header: Some(ReqHeader {
            api_key: ApiKey::Write.into(),
            api_version: ApiVersion::V0.into(),
        }),
        body: Some(body),
    });

    let resp_packet = connection_manager
        .write_send(node_id, req_packet.clone())
        .await?;

    if let JournalEnginePacket::WriteResp(data) = resp_packet {
        resp_header_error(&data.header, req_packet.clone())?;
        if let Some(body) = data.body {
            return Ok(body);
        }
        return Err(JournalClientError::ReceivedPacketNotContainBody(
            req_packet.to_string(),
        ));
    }

    Err(JournalClientError::ReceivedPacketTypeError(
        req_packet.to_string(),
        resp_packet.to_string(),
    ))
}

pub(crate) async fn batch_read(
    connection_manager: &Arc<ConnectionManager>,
    node_id: u64,
    body: ReadReqBody,
) -> Result<ReadRespBody, JournalClientError> {
    let req_packet = JournalEnginePacket::ReadReq(ReadReq {
        header: Some(ReqHeader {
            api_key: ApiKey::Read.into(),
            api_version: ApiVersion::V0.into(),
        }),
        body: Some(body),
    });

    let resp_packet = connection_manager
        .read_send(node_id, req_packet.clone())
        .await?;

    if let JournalEnginePacket::ReadResp(data) = resp_packet {
        resp_header_error(&data.header, req_packet.clone())?;
        if let Some(body) = data.body {
            return Ok(body);
        }
        return Err(JournalClientError::ReceivedPacketNotContainBody(
            req_packet.to_string(),
        ));
    }

    Err(JournalClientError::ReceivedPacketTypeError(
        req_packet.to_string(),
        resp_packet.to_string(),
    ))
}

pub(crate) async fn fetch_offset(
    connection_manager: &Arc<ConnectionManager>,
    node_id: u64,
    body: FetchOffsetReqBody,
) -> Result<FetchOffsetRespBody, JournalClientError> {
    let req_packet = JournalEnginePacket::FetchOffsetReq(FetchOffsetReq {
        header: Some(ReqHeader {
            api_key: ApiKey::FetchOffset.into(),
            api_version: ApiVersion::V0.into(),
        }),
        body: Some(body),
    });

    let resp_packet = connection_manager
        .read_send(node_id, req_packet.clone())
        .await?;

    if let JournalEnginePacket::FetchOffsetResp(data) = resp_packet {
        resp_header_error(&data.header, req_packet.clone())?;
        if let Some(body) = data.body {
            return Ok(body);
        }
        return Err(JournalClientError::ReceivedPacketNotContainBody(
            req_packet.to_string(),
        ));
    }

    Err(JournalClientError::ReceivedPacketTypeError(
        req_packet.to_string(),
        resp_packet.to_string(),
    ))
}
