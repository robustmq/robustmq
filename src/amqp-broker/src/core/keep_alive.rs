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

use common_base::error::ResultCommonError;
use common_base::tools::loop_select_ticket;
use network_server::common::connection_manager::ConnectionManager;
use tokio::sync::broadcast;

use crate::core::cache::AmqpCacheManager;

const GC_INTERVAL_MS: u64 = 30_000;

// AMQP's own heartbeat frame only proves liveness at the protocol layer; the
// underlying TCP disconnect is already detected and removed from
// ConnectionManager generically. This just reconciles AmqpCacheManager against
// that — nothing else notifies it when a connection actually goes away.
#[derive(Clone)]
pub struct AmqpKeepAlive {
    connection_manager: Arc<ConnectionManager>,
    amqp_cache: Arc<AmqpCacheManager>,
}

impl AmqpKeepAlive {
    pub fn new(
        connection_manager: Arc<ConnectionManager>,
        amqp_cache: Arc<AmqpCacheManager>,
    ) -> Self {
        AmqpKeepAlive {
            connection_manager,
            amqp_cache,
        }
    }

    pub async fn start(&self, stop_send: &broadcast::Sender<bool>) {
        let ac_fn = async || -> ResultCommonError { self.tick().await };
        loop_select_ticket(ac_fn, GC_INTERVAL_MS, stop_send).await;
    }

    async fn tick(&self) -> ResultCommonError {
        for connection_id in self.amqp_cache.connection_ids() {
            if self.connection_manager.get_connect(connection_id).is_none() {
                self.amqp_cache.remove_connection(connection_id);
            }
        }
        Ok(())
    }
}
