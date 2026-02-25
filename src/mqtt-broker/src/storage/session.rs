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

use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use dashmap::DashMap;
use grpc_clients::meta::mqtt::call::{
    placement_create_session, placement_delete_session, placement_get_last_will_message,
    placement_list_session, placement_save_last_will_message,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::lastwill::MqttLastWillData;
use metadata_struct::mqtt::session::MqttSession;
use protocol::meta::meta_service_mqtt::{
    CreateSessionRaw, CreateSessionRequest, DeleteSessionRequest, GetLastWillMessageRequest,
    ListSessionRequest, SaveLastWillMessageRequest,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::Instant;
use tracing::{error, info};

const SESSION_BATCH_CHANNEL_SIZE: usize = 5000;
const SESSION_BATCH_SIZE: usize = 100;
const SESSION_BATCH_MAX_WAIT_MS: u64 = 10;

struct SessionBatchItem {
    raw: CreateSessionRaw,
    result_tx: oneshot::Sender<Result<(), CommonError>>,
}

pub struct SessionBatcher {
    sender: mpsc::Sender<SessionBatchItem>,
    consumer: std::sync::Mutex<Option<mpsc::Receiver<SessionBatchItem>>>,
}

impl SessionBatcher {
    pub fn new() -> Arc<Self> {
        let (tx, rx) = mpsc::channel(SESSION_BATCH_CHANNEL_SIZE);
        Arc::new(SessionBatcher {
            sender: tx,
            consumer: std::sync::Mutex::new(Some(rx)),
        })
    }

    pub fn start(&self, client_pool: Arc<ClientPool>) {
        let rx = self
            .consumer
            .lock()
            .unwrap()
            .take()
            .expect("SessionBatcher::start must be called exactly once");
        tokio::spawn(session_batch_consumer(rx, client_pool));
        info!(
            "SessionBatcher started: batch_size={}, max_wait_ms={}",
            SESSION_BATCH_SIZE, SESSION_BATCH_MAX_WAIT_MS
        );
    }

    pub async fn set_session(
        &self,
        client_id: String,
        session: &MqttSession,
    ) -> Result<(), CommonError> {
        let (result_tx, result_rx) = oneshot::channel();
        let raw = CreateSessionRaw {
            client_id,
            session: session.encode()?,
        };

        self.sender
            .send(SessionBatchItem { raw, result_tx })
            .await
            .map_err(|_| CommonError::CommonError("SessionBatcher channel closed".to_string()))?;

        result_rx.await.map_err(|_| {
            CommonError::CommonError("SessionBatcher result channel dropped".to_string())
        })?
    }
}

async fn session_batch_consumer(
    mut rx: mpsc::Receiver<SessionBatchItem>,
    client_pool: Arc<ClientPool>,
) {
    let max_wait = Duration::from_millis(SESSION_BATCH_MAX_WAIT_MS);

    loop {
        let first = match rx.recv().await {
            Some(item) => item,
            None => {
                info!("SessionBatcher channel closed, consumer stopping");
                return;
            }
        };

        let mut batch = vec![first];
        let deadline = Instant::now() + max_wait;

        loop {
            if batch.len() >= SESSION_BATCH_SIZE {
                break;
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            match tokio::time::timeout(remaining, rx.recv()).await {
                Ok(Some(item)) => batch.push(item),
                Ok(None) => {
                    info!("SessionBatcher channel closed during batch collection");
                    flush_batch(batch, &client_pool).await;
                    return;
                }
                Err(_) => break,
            }
        }

        flush_batch(batch, &client_pool).await;
    }
}

async fn flush_batch(batch: Vec<SessionBatchItem>, client_pool: &Arc<ClientPool>) {
    let (raws, txs): (Vec<_>, Vec<_>) = batch
        .into_iter()
        .map(|item| (item.raw, item.result_tx))
        .unzip();

    let config = broker_config();
    let request = CreateSessionRequest { sessions: raws };
    let result =
        placement_create_session(client_pool, &config.get_meta_service_addr(), request).await;

    match result {
        Ok(_) => {
            for tx in txs {
                let _ = tx.send(Ok(()));
            }
        }
        Err(e) => {
            let msg = e.to_string();
            error!("SessionBatcher flush failed: {}", msg);
            for tx in txs {
                let _ = tx.send(Err(CommonError::CommonError(msg.clone())));
            }
        }
    }
}

pub struct SessionStorage {
    client_pool: Arc<ClientPool>,
}

impl SessionStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        SessionStorage { client_pool }
    }

    pub async fn set_session(
        &self,
        client_id: String,
        session: &MqttSession,
    ) -> Result<(), CommonError> {
        let config = broker_config();
        let request = CreateSessionRequest {
            sessions: vec![CreateSessionRaw {
                client_id,
                session: session.encode()?,
            }],
        };

        placement_create_session(&self.client_pool, &config.get_meta_service_addr(), request)
            .await?;
        Ok(())
    }

    pub async fn delete_session(&self, client_id: String) -> Result<(), CommonError> {
        let config = broker_config();
        let request = DeleteSessionRequest { client_id };
        placement_delete_session(&self.client_pool, &config.get_meta_service_addr(), request)
            .await?;
        Ok(())
    }

    pub async fn get_session(&self, client_id: String) -> Result<Option<MqttSession>, CommonError> {
        let config = broker_config();
        let request = ListSessionRequest { client_id };

        let reply =
            placement_list_session(&self.client_pool, &config.get_meta_service_addr(), request)
                .await?;
        if reply.sessions.is_empty() {
            return Ok(None);
        }

        let raw = reply.sessions.first().unwrap();
        let data = MqttSession::decode(raw)?;
        Ok(Some(data))
    }

    pub async fn list_session(
        &self,
        client_id: Option<String>,
    ) -> Result<DashMap<String, MqttSession>, CommonError> {
        let config = broker_config();
        let request = ListSessionRequest {
            client_id: if let Some(id) = client_id {
                id
            } else {
                "".to_string()
            },
        };

        let reply =
            placement_list_session(&self.client_pool, &config.get_meta_service_addr(), request)
                .await?;
        let results = DashMap::with_capacity(2);

        for raw in reply.sessions {
            let data = MqttSession::decode(&raw)?;
            results.insert(data.client_id.clone(), data);
        }

        Ok(results)
    }

    pub async fn save_last_will_message(
        &self,
        client_id: String,
        last_will_message: Vec<u8>,
    ) -> Result<(), CommonError> {
        let config = broker_config();
        let request = SaveLastWillMessageRequest {
            client_id,
            last_will_message,
        };

        placement_save_last_will_message(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;

        Ok(())
    }

    pub async fn get_last_will_message(
        &self,
        client_id: String,
    ) -> Result<Option<MqttLastWillData>, CommonError> {
        let config = broker_config();
        let request = GetLastWillMessageRequest { client_id };

        let reply = placement_get_last_will_message(
            &self.client_pool,
            &config.get_meta_service_addr(),
            request,
        )
        .await?;
        if reply.message.is_empty() {
            return Ok(None);
        }

        let data = MqttLastWillData::decode(&reply.message)?;
        Ok(Some(data))
    }
}
