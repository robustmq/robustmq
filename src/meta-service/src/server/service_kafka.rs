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

use crate::core::notify::{
    send_notify_by_delete_kafka_delegation_token, send_notify_by_delete_kafka_quota,
    send_notify_by_delete_kafka_scram, send_notify_by_set_kafka_delegation_token,
    send_notify_by_set_kafka_quota, send_notify_by_set_kafka_scram,
};
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::kafka::delegation_token::KafkaDelegationTokenStorage;
use crate::storage::kafka::quota::KafkaQuotaStorage;
use crate::storage::kafka::scram::KafkaScramStorage;
use bytes::Bytes;
use metadata_struct::kafka::delegation_token::KafkaDelegationToken;
use metadata_struct::kafka::quota::KafkaClientQuota;
use metadata_struct::kafka::scram::KafkaScramCredential;
use node_call::NodeCallManager;
use prost::Message;
use prost_validate::Validator;
use protocol::meta::meta_service_kafka::kafka_service_server::KafkaService;
use protocol::meta::meta_service_kafka::{
    DeleteKafkaDelegationTokenReply, DeleteKafkaDelegationTokenRequest, DeleteKafkaQuotaReply,
    DeleteKafkaQuotaRequest, DeleteScramCredentialReply, DeleteScramCredentialRequest,
    GetCoordinatorLeaderReply, GetCoordinatorLeaderRequest, ListKafkaDelegationTokenReply,
    ListKafkaDelegationTokenRequest, ListKafkaQuotaReply, ListKafkaQuotaRequest,
    ListScramCredentialReply, ListScramCredentialRequest, SetKafkaDelegationTokenReply,
    SetKafkaDelegationTokenRequest, SetKafkaQuotaReply, SetKafkaQuotaRequest,
    SetScramCredentialReply, SetScramCredentialRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct GrpcKafkaService {
    raft_manager: Arc<MultiRaftManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    call_manager: Arc<NodeCallManager>,
}

impl GrpcKafkaService {
    pub fn new(
        raft_manager: Arc<MultiRaftManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        call_manager: Arc<NodeCallManager>,
    ) -> Self {
        GrpcKafkaService {
            raft_manager,
            rocksdb_engine_handler,
            call_manager,
        }
    }

    fn validate_request<T: Validator>(req: &T) -> Result<(), Status> {
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))
    }

    fn to_status<E: ToString>(e: E) -> Status {
        Status::internal(e.to_string())
    }
}

#[tonic::async_trait]
impl KafkaService for GrpcKafkaService {
    async fn get_coordinator_leader(
        &self,
        _request: Request<GetCoordinatorLeaderRequest>,
    ) -> Result<Response<GetCoordinatorLeaderReply>, Status> {
        // For now the Kafka group coordinator is simply the metadata-raft leader.
        let reply = match self.raft_manager.metadata_leader() {
            Some(leader_node_id) => GetCoordinatorLeaderReply {
                leader_node_id,
                has_leader: true,
            },
            None => GetCoordinatorLeaderReply {
                leader_node_id: 0,
                has_leader: false,
            },
        };
        Ok(Response::new(reply))
    }

    async fn set_kafka_quota(
        &self,
        request: Request<SetKafkaQuotaRequest>,
    ) -> Result<Response<SetKafkaQuotaReply>, Status> {
        let req = request.into_inner();
        Self::validate_request(&req)?;
        let quota = KafkaClientQuota::decode(&req.quota).map_err(Self::to_status)?;

        let data = StorageData::new(
            StorageDataType::KafkaSetQuota,
            Bytes::from(req.encode_to_vec()),
        );
        self.raft_manager
            .write_metadata(data)
            .await
            .map_err(Self::to_status)?;

        send_notify_by_set_kafka_quota(&self.call_manager, quota)
            .await
            .map_err(Self::to_status)?;

        Ok(Response::new(SetKafkaQuotaReply {}))
    }

    async fn delete_kafka_quota(
        &self,
        request: Request<DeleteKafkaQuotaRequest>,
    ) -> Result<Response<DeleteKafkaQuotaReply>, Status> {
        let req = request.into_inner();
        Self::validate_request(&req)?;

        let storage = KafkaQuotaStorage::new(self.rocksdb_engine_handler.clone());
        let existing = storage
            .get(&req.tenant, &req.entity_type, &req.entity_name)
            .map_err(Self::to_status)?;

        let data = StorageData::new(
            StorageDataType::KafkaDeleteQuota,
            Bytes::from(req.encode_to_vec()),
        );
        self.raft_manager
            .write_metadata(data)
            .await
            .map_err(Self::to_status)?;

        if let Some(quota) = existing {
            send_notify_by_delete_kafka_quota(&self.call_manager, quota)
                .await
                .map_err(Self::to_status)?;
        }

        Ok(Response::new(DeleteKafkaQuotaReply {}))
    }

    async fn list_kafka_quota(
        &self,
        request: Request<ListKafkaQuotaRequest>,
    ) -> Result<Response<ListKafkaQuotaReply>, Status> {
        let req = request.into_inner();
        Self::validate_request(&req)?;

        let storage = KafkaQuotaStorage::new(self.rocksdb_engine_handler.clone());
        let quotas = storage
            .list_by_tenant(&req.tenant)
            .map_err(Self::to_status)?;
        let mut encoded = Vec::with_capacity(quotas.len());
        for quota in quotas {
            encoded.push(quota.encode().map_err(Self::to_status)?);
        }

        Ok(Response::new(ListKafkaQuotaReply { quotas: encoded }))
    }

    async fn set_kafka_delegation_token(
        &self,
        request: Request<SetKafkaDelegationTokenRequest>,
    ) -> Result<Response<SetKafkaDelegationTokenReply>, Status> {
        let req = request.into_inner();
        Self::validate_request(&req)?;
        let token = KafkaDelegationToken::decode(&req.token).map_err(Self::to_status)?;

        let data = StorageData::new(
            StorageDataType::KafkaSetDelegationToken,
            Bytes::from(req.encode_to_vec()),
        );
        self.raft_manager
            .write_metadata(data)
            .await
            .map_err(Self::to_status)?;

        send_notify_by_set_kafka_delegation_token(&self.call_manager, token)
            .await
            .map_err(Self::to_status)?;

        Ok(Response::new(SetKafkaDelegationTokenReply {}))
    }

    async fn delete_kafka_delegation_token(
        &self,
        request: Request<DeleteKafkaDelegationTokenRequest>,
    ) -> Result<Response<DeleteKafkaDelegationTokenReply>, Status> {
        let req = request.into_inner();
        Self::validate_request(&req)?;

        let data = StorageData::new(
            StorageDataType::KafkaDeleteDelegationToken,
            Bytes::from(req.encode_to_vec()),
        );
        self.raft_manager
            .write_metadata(data)
            .await
            .map_err(Self::to_status)?;

        send_notify_by_delete_kafka_delegation_token(&self.call_manager, req.token_id.clone())
            .await
            .map_err(Self::to_status)?;

        Ok(Response::new(DeleteKafkaDelegationTokenReply {}))
    }

    async fn list_kafka_delegation_token(
        &self,
        request: Request<ListKafkaDelegationTokenRequest>,
    ) -> Result<Response<ListKafkaDelegationTokenReply>, Status> {
        let req = request.into_inner();
        Self::validate_request(&req)?;

        let storage = KafkaDelegationTokenStorage::new(self.rocksdb_engine_handler.clone());
        let tokens = storage
            .list_by_tenant(&req.tenant)
            .map_err(Self::to_status)?;
        let mut encoded = Vec::with_capacity(tokens.len());
        for token in tokens {
            encoded.push(token.encode().map_err(Self::to_status)?);
        }

        Ok(Response::new(ListKafkaDelegationTokenReply {
            tokens: encoded,
        }))
    }

    async fn set_scram_credential(
        &self,
        request: Request<SetScramCredentialRequest>,
    ) -> Result<Response<SetScramCredentialReply>, Status> {
        let req = request.into_inner();
        Self::validate_request(&req)?;
        let credential = KafkaScramCredential::decode(&req.credential).map_err(Self::to_status)?;

        let data = StorageData::new(
            StorageDataType::KafkaSetScram,
            Bytes::from(req.encode_to_vec()),
        );
        self.raft_manager
            .write_metadata(data)
            .await
            .map_err(Self::to_status)?;

        send_notify_by_set_kafka_scram(&self.call_manager, credential)
            .await
            .map_err(Self::to_status)?;

        Ok(Response::new(SetScramCredentialReply {}))
    }

    async fn delete_scram_credential(
        &self,
        request: Request<DeleteScramCredentialRequest>,
    ) -> Result<Response<DeleteScramCredentialReply>, Status> {
        let req = request.into_inner();
        Self::validate_request(&req)?;

        let storage = KafkaScramStorage::new(self.rocksdb_engine_handler.clone());
        let existing = storage
            .get(&req.tenant, &req.user, req.mechanism as i8)
            .map_err(Self::to_status)?;

        let data = StorageData::new(
            StorageDataType::KafkaDeleteScram,
            Bytes::from(req.encode_to_vec()),
        );
        self.raft_manager
            .write_metadata(data)
            .await
            .map_err(Self::to_status)?;

        if let Some(credential) = existing {
            send_notify_by_delete_kafka_scram(&self.call_manager, credential)
                .await
                .map_err(Self::to_status)?;
        }

        Ok(Response::new(DeleteScramCredentialReply {}))
    }

    async fn list_scram_credential(
        &self,
        request: Request<ListScramCredentialRequest>,
    ) -> Result<Response<ListScramCredentialReply>, Status> {
        let req = request.into_inner();
        Self::validate_request(&req)?;

        let storage = KafkaScramStorage::new(self.rocksdb_engine_handler.clone());
        let credentials = storage
            .list_by_tenant(&req.tenant)
            .map_err(Self::to_status)?;
        let mut encoded = Vec::with_capacity(credentials.len());
        for credential in credentials {
            encoded.push(credential.encode().map_err(Self::to_status)?);
        }

        Ok(Response::new(ListScramCredentialReply {
            credentials: encoded,
        }))
    }
}
