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

use bytes::Bytes;
use metadata_struct::kafka::delegation_token::KafkaDelegationToken;
use metadata_struct::kafka::quota::KafkaClientQuota;
use metadata_struct::kafka::scram::KafkaScramCredential;
use prost::Message;
use protocol::meta::meta_service_kafka::{
    DeleteKafkaDelegationTokenRequest, DeleteKafkaQuotaRequest, DeleteScramCredentialRequest,
    SetKafkaDelegationTokenRequest, SetKafkaQuotaRequest, SetScramCredentialRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;

use crate::core::error::MetaServiceError;
use crate::storage::kafka::delegation_token::KafkaDelegationTokenStorage;
use crate::storage::kafka::quota::KafkaQuotaStorage;
use crate::storage::kafka::scram::KafkaScramStorage;

#[derive(Clone)]
pub struct DataRouteKafka {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl DataRouteKafka {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        DataRouteKafka {
            rocksdb_engine_handler,
        }
    }

    pub fn set_quota(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = SetKafkaQuotaRequest::decode(value.as_ref())?;
        let quota = KafkaClientQuota::decode(&req.quota)?;
        let storage = KafkaQuotaStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(quota)?;
        Ok(())
    }

    pub fn delete_quota(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteKafkaQuotaRequest::decode(value.as_ref())?;
        let storage = KafkaQuotaStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.tenant, &req.entity_type, &req.entity_name)?;
        Ok(())
    }

    pub fn set_delegation_token(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = SetKafkaDelegationTokenRequest::decode(value.as_ref())?;
        let token = KafkaDelegationToken::decode(&req.token)?;
        let storage = KafkaDelegationTokenStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(token)?;
        Ok(())
    }

    pub fn delete_delegation_token(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteKafkaDelegationTokenRequest::decode(value.as_ref())?;
        let storage = KafkaDelegationTokenStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.tenant, &req.token_id)?;
        Ok(())
    }

    pub fn set_scram_credential(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = SetScramCredentialRequest::decode(value.as_ref())?;
        let credential = KafkaScramCredential::decode(&req.credential)?;
        let storage = KafkaScramStorage::new(self.rocksdb_engine_handler.clone());
        storage.save(credential)?;
        Ok(())
    }

    pub fn delete_scram_credential(&self, value: Bytes) -> Result<(), MetaServiceError> {
        let req = DeleteScramCredentialRequest::decode(value.as_ref())?;
        let storage = KafkaScramStorage::new(self.rocksdb_engine_handler.clone());
        storage.delete(&req.tenant, &req.user, req.mechanism as i8)?;
        Ok(())
    }
}
