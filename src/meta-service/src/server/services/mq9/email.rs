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
use crate::core::notify::{send_notify_by_create_mq9_email, send_notify_by_delete_mq9_mail};
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::mq9::email::Mq9EmailStorage;
use common_base::utils::serialize::encode_to_bytes;
use metadata_struct::mq9::email::MQ9Email;
use node_call::NodeCallManager;
use protocol::meta::meta_service_mq9::{
    CreateEmailReply, CreateEmailRequest, DeleteEmailReply, DeleteEmailRequest, ListEmailReply,
    ListEmailRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::pin::Pin;
use std::sync::Arc;
use tonic::codegen::tokio_stream::Stream;
use tonic::Status;

pub type ListEmailStream =
    Result<Pin<Box<dyn Stream<Item = Result<ListEmailReply, Status>> + Send>>, MetaServiceError>;

pub fn list_email_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListEmailRequest,
) -> ListEmailStream {
    let storage = Mq9EmailStorage::new(rocksdb_engine_handler.clone());
    let emails: Vec<MQ9Email> = if !req.tenant.is_empty() {
        storage.list_by_tenant(&req.tenant)?
    } else {
        storage.list()?
    };

    let output = async_stream::try_stream! {
        for email in emails {
            yield ListEmailReply { email: email.encode()? };
        }
    };

    Ok(Box::pin(output))
}

pub async fn create_email_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &CreateEmailRequest,
) -> Result<CreateEmailReply, MetaServiceError> {
    let email = MQ9Email::decode(&req.content)?;

    // Idempotency: if the mailbox already exists, return success silently.
    let storage = Mq9EmailStorage::new(rocksdb_engine_handler.clone());
    if storage.get(&req.tenant, &email.mail_id)?.is_some() {
        return Ok(CreateEmailReply {});
    }

    let data = StorageData::new(StorageDataType::Mq9CreateEmail, encode_to_bytes(req));
    raft_manager.write_data(&req.tenant, data).await?;

    send_notify_by_create_mq9_email(call_manager, email).await?;

    Ok(CreateEmailReply {})
}

pub async fn delete_email_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &DeleteEmailRequest,
) -> Result<DeleteEmailReply, MetaServiceError> {
    let storage = Mq9EmailStorage::new(rocksdb_engine_handler.clone());
    if let Some(email) = storage.get(&req.tenant, &req.mail_id)? {
        let data = StorageData::new(StorageDataType::Mq9DeleteEmail, encode_to_bytes(req));
        raft_manager.write_data(&req.tenant, data).await?;
        send_notify_by_delete_mq9_mail(call_manager, email).await?;
    }
    Ok(DeleteEmailReply {})
}
