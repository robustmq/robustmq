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
use crate::core::notify::{
    send_notify_by_add_nats_subscribe, send_notify_by_delete_nats_subscribe,
};
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::nats::NatsSubscribeStorage;
use common_base::utils::serialize::encode_to_bytes;
use metadata_struct::nats::subscribe::NatsSubscribe;
use node_call::NodeCallManager;
use protocol::meta::meta_service_nats::{
    CreateNatsSubscribeReply, CreateNatsSubscribeRequest, DeleteNatsSubscribeReply,
    DeleteNatsSubscribeRequest, ListNatsSubscribeReply, ListNatsSubscribeRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::pin::Pin;
use std::sync::Arc;
use tonic::codegen::tokio_stream::Stream;
use tonic::Status;

pub type ListNatsSubscribeStream = Result<
    Pin<Box<dyn Stream<Item = Result<ListNatsSubscribeReply, Status>> + Send>>,
    MetaServiceError,
>;

pub fn list_nats_subscribe_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListNatsSubscribeRequest,
) -> ListNatsSubscribeStream {
    let storage = NatsSubscribeStorage::new(rocksdb_engine_handler.clone());
    let subscribes: Vec<NatsSubscribe> = if req.connect_id != 0 {
        storage
            .list()?
            .into_iter()
            .filter(|s| s.connect_id == req.connect_id)
            .collect()
    } else {
        storage.list()?
    };

    let output = async_stream::try_stream! {
        for subscribe in subscribes {
            yield ListNatsSubscribeReply { subscribe: subscribe.encode()? };
        }
    };

    Ok(Box::pin(output))
}

pub async fn create_nats_subscribe_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &CreateNatsSubscribeRequest,
) -> Result<CreateNatsSubscribeReply, MetaServiceError> {
    let storage = NatsSubscribeStorage::new(rocksdb_engine_handler.clone());
    let mut new_items: Vec<Vec<u8>> = Vec::new();
    for item in &req.subscribes {
        let subscribe = NatsSubscribe::decode(item)?;
        if storage
            .get(subscribe.broker_id, subscribe.connect_id, &subscribe.sid)?
            .is_none()
        {
            new_items.push(item.clone());
        }
    }
    if new_items.is_empty() {
        return Ok(CreateNatsSubscribeReply {});
    }
    let new_req = CreateNatsSubscribeRequest {
        subscribes: new_items.clone(),
    };
    let data = StorageData::new(StorageDataType::NatsSetSubscribe, encode_to_bytes(&new_req));
    raft_manager.write_data("nats/subscribe", data).await?;
    for item in &new_items {
        let subscribe = NatsSubscribe::decode(item)?;
        send_notify_by_add_nats_subscribe(call_manager, subscribe).await?;
    }
    Ok(CreateNatsSubscribeReply {})
}

pub async fn delete_nats_subscribe_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &DeleteNatsSubscribeRequest,
) -> Result<DeleteNatsSubscribeReply, MetaServiceError> {
    let storage = NatsSubscribeStorage::new(rocksdb_engine_handler.clone());
    let mut existing: Vec<NatsSubscribe> = Vec::new();
    for key in &req.keys {
        if let Some(subscribe) = storage.get(key.broker_id, key.connect_id, &key.sid)? {
            existing.push(subscribe);
        }
    }
    if existing.is_empty() {
        return Ok(DeleteNatsSubscribeReply {});
    }
    let data = StorageData::new(StorageDataType::NatsDeleteSubscribe, encode_to_bytes(req));
    raft_manager.write_data("nats/subscribe", data).await?;
    for subscribe in existing {
        send_notify_by_delete_nats_subscribe(call_manager, subscribe).await?;
    }
    Ok(DeleteNatsSubscribeReply {})
}
