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

use crate::core::error::MetaServiceError;
use crate::core::notify::{send_notify_by_add_nats_subject, send_notify_by_delete_nats_subject};
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::nats::NatsSubjectStorage;
use common_base::utils::serialize::encode_to_bytes;
use metadata_struct::nats::subject::NatsSubject;
use node_call::NodeCallManager;
use protocol::meta::meta_service_nats::{
    CreateNatsSubjectReply, CreateNatsSubjectRequest, DeleteNatsSubjectReply,
    DeleteNatsSubjectRequest, ListNatsSubjectReply, ListNatsSubjectRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::pin::Pin;
use std::sync::Arc;
use tonic::codegen::tokio_stream::Stream;
use tonic::Status;

type ListNatsSubjectStream = Result<
    Pin<Box<dyn Stream<Item = Result<ListNatsSubjectReply, Status>> + Send>>,
    MetaServiceError,
>;

pub fn list_nats_subject_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListNatsSubjectRequest,
) -> ListNatsSubjectStream {
    let storage = NatsSubjectStorage::new(rocksdb_engine_handler.clone());
    let subjects: Vec<NatsSubject> = if !req.name.is_empty() {
        storage.get(&req.tenant, &req.name)?.into_iter().collect()
    } else if !req.tenant.is_empty() {
        storage.list_by_tenant(&req.tenant)?
    } else {
        storage.list()?
    };

    let output = async_stream::try_stream! {
        for subject in subjects {
            yield ListNatsSubjectReply { subject: subject.encode()? };
        }
    };

    Ok(Box::pin(output))
}

pub async fn create_nats_subject_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    req: &CreateNatsSubjectRequest,
) -> Result<CreateNatsSubjectReply, MetaServiceError> {
    let subject = NatsSubject::decode(&req.subject)?;
    let data = StorageData::new(StorageDataType::NatsSetSubject, encode_to_bytes(req));
    raft_manager.write_data(&subject.name, data).await?;
    send_notify_by_add_nats_subject(call_manager, subject).await?;
    Ok(CreateNatsSubjectReply {})
}

pub async fn delete_nats_subject_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &DeleteNatsSubjectRequest,
) -> Result<DeleteNatsSubjectReply, MetaServiceError> {
    let storage = NatsSubjectStorage::new(rocksdb_engine_handler.clone());
    let subject = match storage.get(&req.tenant, &req.name)? {
        Some(s) => s,
        None => return Ok(DeleteNatsSubjectReply {}),
    };

    let data = StorageData::new(StorageDataType::NatsDeleteSubject, encode_to_bytes(req));
    raft_manager.write_data(&req.name, data).await?;
    send_notify_by_delete_nats_subject(call_manager, subject).await?;
    Ok(DeleteNatsSubjectReply {})
}
