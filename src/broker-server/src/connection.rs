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

use common_base::{error::ResultCommonError, tools::loop_select_ticket};
use network_server::common::connection_manager::ConnectionManager;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

pub async fn network_connection_gc(
    connection_manager: Arc<ConnectionManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        connection_manager.connection_gc().await;
        Ok(())
    };

    info!("Network connection recovery thread has been successfully started.");
    loop_select_ticket(ac_fn, 3, &stop_send).await;
}
