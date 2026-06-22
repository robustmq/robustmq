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
use std::time::Duration;

use admin_server::client::AdminHttpClient;
use admin_server::engine::segment::{SegmentListReq, SegmentListResp, SegmentListRespRaw};
use admin_server::engine::shard::{ShardCreateReq, ShardListReq, ShardListRow};
use broker_core::cache::NodeCacheManager;
use common_base::http_response::AdminServerResponse;
use common_config::config::BrokerConfig;
use grpc_clients::pool::ClientPool;
use metadata_struct::meta::node::BrokerNode;
use protocol::storage::codec::StorageEnginePacket;
use protocol::storage::protocol::{
    ReadReq, ReadReqBody, ReadReqFilter, ReadReqMessage, ReadReqOptions, ReadType, WriteReq,
    WriteReqBody,
};
use storage_engine::clients::manager::ClientConnectionManager;
use storage_engine::core::cache::StorageCacheManager;
use tokio::time::sleep;

pub const ADMIN_ADDR: &str = "http://127.0.0.1:58080";
pub const ENGINE_NODE_ID: u64 = 1;
pub const ENGINE_ADDR: &str = "127.0.0.1:1779";

pub fn admin_client() -> AdminHttpClient {
    AdminHttpClient::new(ADMIN_ADDR)
}

pub fn engine_client() -> Arc<ClientConnectionManager> {
    let broker_cache = Arc::new(NodeCacheManager::new(BrokerConfig::default()));
    broker_cache.add_node(BrokerNode {
        node_id: ENGINE_NODE_ID,
        engine_addr: ENGINE_ADDR.to_string(),
        ..Default::default()
    });
    let cache = Arc::new(StorageCacheManager::new(broker_cache));
    Arc::new(ClientConnectionManager::new(cache, 2))
}

pub fn grpc_pool() -> ClientPool {
    ClientPool::new(2)
}

pub fn meta_addr() -> String {
    crate::common::get_placement_addr()
}

pub async fn create_shard(client: &AdminHttpClient, shard_name: &str, config: &str) {
    let resp = client
        .create_shard(&ShardCreateReq {
            shard_name: shard_name.to_string(),
            topic_name: None,
            desc: None,
            config: config.to_string(),
        })
        .await
        .expect("create_shard http failed");
    let v: AdminServerResponse<serde_json::Value> = serde_json::from_str(&resp).unwrap();
    assert_eq!(v.code, 0, "create_shard failed: {:?}", v.error);
    sleep(Duration::from_secs(3)).await;
}

pub async fn poll_until<F, Fut>(timeout: Duration, interval: Duration, mut check: F) -> bool
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if check().await {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        sleep(interval).await;
    }
}

pub async fn get_shard_list(client: &AdminHttpClient, shard_name: &str) -> Vec<ShardListRow> {
    let req = ShardListReq {
        shard_name: Some(shard_name.to_string()),
        ..Default::default()
    };
    client
        .get_shard_list::<_, Vec<ShardListRow>>(&req)
        .await
        .expect("get_shard_list failed")
        .data
}

/// Send messages and return the number written. Panics on any protocol or engine error.
pub async fn write_messages(
    conn: &Arc<ClientConnectionManager>,
    shard_name: &str,
    messages: Vec<Vec<u8>>,
) -> usize {
    let count = messages.len();
    let req = WriteReq::new(WriteReqBody::new(shard_name.to_string(), messages));
    let resp = conn
        .write_send(ENGINE_NODE_ID, StorageEnginePacket::WriteReq(req))
        .await
        .expect("write_send failed");
    match resp {
        StorageEnginePacket::WriteResp(r) => {
            if let Some(err) = r.header.error {
                panic!("WriteResp error: {}:{}", err.code, err.error);
            }
            assert_eq!(r.body.status[0].messages.len(), count);
            count
        }
        other => panic!("expected WriteResp, got {}", other),
    }
}

/// Send a read request and return raw message bytes. Panics on any protocol or engine error.
pub async fn read_messages_raw(
    conn: &Arc<ClientConnectionManager>,
    shard_name: &str,
    read_type: ReadType,
    filter: ReadReqFilter,
    max_record: u64,
) -> Vec<Vec<u8>> {
    let req = ReadReq::new(ReadReqBody::new(vec![ReadReqMessage::new(
        shard_name.to_string(),
        read_type,
        false,
        filter,
        ReadReqOptions::new(1024 * 1024, max_record),
    )]));
    let resp = conn
        .read_send(ENGINE_NODE_ID, StorageEnginePacket::ReadReq(req))
        .await
        .expect("read_send failed");
    match resp {
        StorageEnginePacket::ReadResp(r) => {
            if let Some(err) = r.header.error {
                panic!("ReadResp error: {}:{}", err.code, err.error);
            }
            r.body.messages
        }
        other => panic!("expected ReadResp, got {}", other),
    }
}

pub async fn get_segment_list(
    client: &AdminHttpClient,
    shard_name: &str,
) -> Vec<SegmentListRespRaw> {
    let req = SegmentListReq {
        shard_name: shard_name.to_string(),
    };
    let raw = client
        .get_segment_list(&req)
        .await
        .expect("get_segment_list failed");
    let resp: AdminServerResponse<SegmentListResp> =
        serde_json::from_str(&raw).expect("parse segment list failed");
    assert_eq!(resp.code, 0, "segment list error: {:?}", resp.error);
    resp.data.segment_list
}
