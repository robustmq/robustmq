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
use crate::core::notify::{send_notify_by_create_mq9_mail, send_notify_by_delete_mq9_mail};
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::mq9::mail::Mq9MailStorage;
use common_base::utils::serialize::encode_to_bytes;
use metadata_struct::mq9::mail::MQ9Mail;
use node_call::NodeCallManager;
use protocol::meta::meta_service_mq9::{
    CreateMailReply, CreateMailRequest, DeleteMailReply, DeleteMailRequest, ListMailReply,
    ListMailRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::pin::Pin;
use std::sync::Arc;
use tonic::codegen::tokio_stream::Stream;
use tonic::Status;

pub type ListMailStream =
    Result<Pin<Box<dyn Stream<Item = Result<ListMailReply, Status>> + Send>>, MetaServiceError>;

pub fn list_mail_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListMailRequest,
) -> ListMailStream {
    let storage = Mq9MailStorage::new(rocksdb_engine_handler.clone());
    let mails: Vec<MQ9Mail> = if !req.tenant.is_empty() {
        storage.list_by_tenant(&req.tenant)?
    } else {
        storage.list()?
    };

    let output = async_stream::try_stream! {
        for mail in mails {
            yield ListMailReply { mail: mail.encode()? };
        }
    };

    Ok(Box::pin(output))
}

pub async fn create_mail_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &CreateMailRequest,
) -> Result<CreateMailReply, MetaServiceError> {
    let mail = MQ9Mail::decode(&req.content)?;

    // Idempotency: if the mailbox already exists, return success silently.
    let storage = Mq9MailStorage::new(rocksdb_engine_handler.clone());
    if storage.get(&req.tenant, &mail.mail_address)?.is_some() {
        return Ok(CreateMailReply {});
    }

    let data = StorageData::new(StorageDataType::Mq9CreateMail, encode_to_bytes(req));
    raft_manager.write_data(&req.tenant, data).await?;

    send_notify_by_create_mq9_mail(call_manager, mail).await?;

    Ok(CreateMailReply {})
}

pub async fn delete_mail_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &DeleteMailRequest,
) -> Result<DeleteMailReply, MetaServiceError> {
    let storage = Mq9MailStorage::new(rocksdb_engine_handler.clone());
    if let Some(mail) = storage.get(&req.tenant, &req.mail_address)? {
        let data = StorageData::new(StorageDataType::Mq9DeleteMail, encode_to_bytes(req));
        raft_manager.write_data(&req.tenant, data).await?;
        send_notify_by_delete_mq9_mail(call_manager, mail).await?;
    }
    Ok(DeleteMailReply {})
}
