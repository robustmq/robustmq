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

use crate::controller::call_broker::call::BrokerCallManager;
use crate::controller::engine_gc::start_engine_delete_gc_thread;
use crate::controller::session_expire::ExpireLastWill;
use crate::core::cache::CacheManager;
use crate::raft::manager::MultiRaftManager;
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select_ticket, now_second};
use grpc_clients::pool::ClientPool;
use message_expire::MessageExpire;
use rocksdb_engine::rocksdb::RocksDBEngine;
use session_expire::SessionExpire;
use std::sync::Arc;
use tokio::sync::broadcast;

pub mod call_broker;
pub mod connector;
pub mod engine_gc;
pub mod message_expire;
pub mod session_expire;

pub fn is_send_last_will(lastwill: &ExpireLastWill) -> bool {
    if now_second() >= lastwill.delay_sec {
        return true;
    }
    false
}

pub struct BrokerController {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    call_manager: Arc<BrokerCallManager>,
    raft_manager: Arc<MultiRaftManager>,
    cache_manager: Arc<CacheManager>,
    client_pool: Arc<ClientPool>,
    stop_send: broadcast::Sender<bool>,
}

impl BrokerController {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        raft_manager: Arc<MultiRaftManager>,
        cache_manager: Arc<CacheManager>,
        call_manager: Arc<BrokerCallManager>,
        client_pool: Arc<ClientPool>,
        stop_send: broadcast::Sender<bool>,
    ) -> BrokerController {
        BrokerController {
            rocksdb_engine_handler,
            cache_manager,
            call_manager,
            raft_manager,
            client_pool,
            stop_send,
        }
    }

    pub async fn start(&self) {
        // Periodically check if the session has expired
        let session = SessionExpire::new(
            self.rocksdb_engine_handler.clone(),
            self.cache_manager.clone(),
            self.client_pool.clone(),
        );
        let stop_send = self.stop_send.clone();
        tokio::spawn(Box::pin(async move {
            let ac_fn = async || -> ResultCommonError {
                session.session_expire().await;
                Ok(())
            };
            loop_select_ticket(ac_fn, 1000, &stop_send).await;
        }));

        // Periodically check if the session has expired
        let session = SessionExpire::new(
            self.rocksdb_engine_handler.clone(),
            self.cache_manager.clone(),
            self.client_pool.clone(),
        );

        let stop_send = self.stop_send.clone();
        tokio::spawn(Box::pin(async move {
            let ac_fn = async || -> ResultCommonError {
                session.last_will_expire_send().await;
                Ok(())
            };
            loop_select_ticket(ac_fn, 1000, &stop_send).await;
        }));

        // Whether the timed message expires
        let message = MessageExpire::new(self.rocksdb_engine_handler.clone());
        let stop_send = self.stop_send.clone();
        tokio::spawn(Box::pin(async move {
            let ac_fn = async || -> ResultCommonError {
                message.retain_message_expire().await;
                Ok(())
            };
            loop_select_ticket(ac_fn, 1000, &stop_send).await;
        }));

        // Periodically detects whether a will message is sent
        let message = MessageExpire::new(self.rocksdb_engine_handler.clone());
        let stop_send = self.stop_send.clone();
        tokio::spawn(Box::pin(async move {
            let ac_fn = async || -> ResultCommonError {
                message.last_will_message_expire().await;
                Ok(())
            };
            loop_select_ticket(ac_fn, 1000, &stop_send).await;
        }));

        // storage engine gc
        let raft_manager = self.raft_manager.clone();
        let cache_manager = self.cache_manager.clone();
        let call_manager = self.call_manager.clone();
        let client_pool = self.client_pool.clone();
        let stop_send = self.stop_send.clone();
        tokio::spawn(Box::pin(async move {
            start_engine_delete_gc_thread(
                raft_manager,
                cache_manager,
                call_manager,
                client_pool,
                stop_send,
            )
            .await;
        }));
    }
}
