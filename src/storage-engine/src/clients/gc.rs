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

use crate::clients::manager::ClientConnectionManager;
use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tracing::debug;

pub const CONNECTION_IDLE_TIMEOUT_SECS: u64 = 600;
const GC_INTERVAL_MILLIS: u64 = 10000;

pub fn start_conn_gc_thread(
    connection_manager: Arc<ClientConnectionManager>,
    stop_send: Sender<bool>,
) {
    tokio::spawn(async move {
        let ac_fn = async || -> ResultCommonError {
            gc_conn(&connection_manager).await;
            Ok(())
        };
        loop_select_ticket(ac_fn, GC_INTERVAL_MILLIS, &stop_send).await;
    });
}

async fn gc_conn(connection_manager: &ClientConnectionManager) {
    let inactive_conns = connection_manager.get_inactive_conn().await;
    if !inactive_conns.is_empty() {
        debug!(
            "Found {} inactive connections to clean up",
            inactive_conns.len()
        );
    }

    for (node_id, seq, conn_type) in inactive_conns {
        connection_manager.close_conn(node_id, seq, conn_type).await;
    }
}
