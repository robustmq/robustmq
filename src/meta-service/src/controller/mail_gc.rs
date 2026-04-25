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

use crate::core::notify::send_notify_by_delete_mq9_mail;
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::mq9::mail::Mq9MailStorage;
use bytes::Bytes;
use common_base::error::common::CommonError;
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select_ticket, now_second};
use node_call::NodeCallManager;
use prost::Message as _;
use protocol::meta::meta_service_mq9::DeleteMailRequest;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

// Scan every 5 minutes
const MAIL_GC_INTERVAL_MS: u64 = 60 * 1000;

pub async fn start_mail_gc_thread(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    raft_manager: Arc<MultiRaftManager>,
    node_call_manager: Arc<NodeCallManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        if let Err(e) =
            gc_expired_mails(&rocksdb_engine_handler, &raft_manager, &node_call_manager).await
        {
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    };
    loop_select_ticket(ac_fn, MAIL_GC_INTERVAL_MS, &stop_send).await;
}

async fn gc_expired_mails(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    raft_manager: &Arc<MultiRaftManager>,
    node_call_manager: &Arc<NodeCallManager>,
) -> Result<(), CommonError> {
    let storage = Mq9MailStorage::new(rocksdb_engine_handler.clone());
    let all_mails = storage.list()?;
    let now = now_second();

    for mail in all_mails {
        // ttl == 0 means no expiry; skip.
        if mail.ttl == 0 {
            continue;
        }

        let elapsed = now.saturating_sub(mail.create_time);
        if elapsed < mail.ttl {
            continue;
        }

        // Delete via raft so all nodes apply the same deletion.
        let req = DeleteMailRequest {
            tenant: mail.tenant.clone(),
            mail_address: mail.mail_address.clone(),
        };
        let data = StorageData::new(
            StorageDataType::Mq9DeleteMail,
            Bytes::from(req.encode_to_vec()),
        );
        if let Err(e) = raft_manager.write_data(&mail.mail_address, data).await {
            warn!(
                "Failed to delete expired mail via raft: tenant={}, mail_address={}, error={}",
                mail.tenant, mail.mail_address, e
            );
            continue;
        }

        // Notify broker nodes to evict from in-memory cache.
        if let Err(e) = send_notify_by_delete_mq9_mail(node_call_manager, mail.clone()).await {
            warn!(
                "Failed to notify brokers to delete mail: tenant={}, mail_address={}, error={}",
                mail.tenant, mail.mail_address, e
            );
        }

        info!(
            "Mail {} cleaned up successfully: tenant={}, create_time={}s ago, ttl={}s",
            mail.mail_address, mail.tenant, elapsed, mail.ttl
        );
    }

    Ok(())
}
