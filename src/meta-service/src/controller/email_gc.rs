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

use crate::core::notify::send_notify_by_delete_mq9_email;
use crate::storage::mq9::email::Mq9EmailStorage;
use common_base::error::common::CommonError;
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select_ticket, now_second};
use node_call::NodeCallManager;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

// Scan every 5 minutes
const EMAIL_GC_INTERVAL_MS: u64 = 60 * 1000;

pub async fn start_email_gc_thread(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    node_call_manager: Arc<NodeCallManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        if let Err(e) = gc_expired_emails(&rocksdb_engine_handler, &node_call_manager).await {
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    };
    loop_select_ticket(ac_fn, EMAIL_GC_INTERVAL_MS, &stop_send).await;
}

async fn gc_expired_emails(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    node_call_manager: &Arc<NodeCallManager>,
) -> Result<(), CommonError> {
    let storage = Mq9EmailStorage::new(rocksdb_engine_handler.clone());
    let all_emails = storage.list()?;
    let now = now_second();

    for email in all_emails {
        // ttl == 0 means no expiry; skip.
        if email.ttl == 0 {
            continue;
        }

        let elapsed = now.saturating_sub(email.create_time);
        if elapsed < email.ttl {
            continue;
        }

        // Delete from RocksDB.
        if let Err(e) = storage.delete(&email.tenant, &email.mail_id) {
            warn!(
                "Failed to delete expired email: tenant={}, mail_id={}, error={}",
                email.tenant, email.mail_id, e
            );
            continue;
        }

        // Notify broker nodes to evict from in-memory cache.
        if let Err(e) = send_notify_by_delete_mq9_email(node_call_manager, email.clone()).await {
            warn!(
                "Failed to notify brokers to delete email: tenant={}, mail_id={}, error={}",
                email.tenant, email.mail_id, e
            );
        }

        info!(
            "Email {} cleaned up successfully: tenant={}, create_time={}s ago, ttl={}s",
            email.mail_id, email.tenant, elapsed, email.ttl
        );
    }

    Ok(())
}
