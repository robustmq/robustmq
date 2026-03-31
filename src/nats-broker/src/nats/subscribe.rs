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

use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
use metadata_struct::tenant::DEFAULT_TENANT;
use network_server::common::connection_manager::ConnectionManager;
use protocol::nats::packet::NatsPacket;
use protocol::robust::{
    NatsWrapperExtend, RobustMQPacket, RobustMQPacketWrapper, RobustMQProtocol,
    RobustMQWrapperExtend,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::driver::StorageDriverManager;
use tokio::time::sleep;
use tracing::error;

pub fn process_sub(
    connection_id: u64,
    subject: &str,
    _queue_group: Option<&str>,
    sid: &str,
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
) -> Option<NatsPacket> {
    let subject = subject.to_string();
    let sid = sid.to_string();
    let read_config = AdapterReadConfig::new();

    tokio::spawn(async move {
        // key: shard_name (matches StorageRecord.metadata.shard), value: next offset to read
        let mut shard_offsets: HashMap<String, u64> = HashMap::new();
        loop {
            match storage_driver_manager
                .read_by_offset(DEFAULT_TENANT, &subject, &shard_offsets, &read_config)
                .await
            {
                Ok(records) if records.is_empty() => {
                    sleep(Duration::from_millis(100)).await;
                }
                Ok(records) => {
                    for record in &records {
                        shard_offsets
                            .insert(record.metadata.shard.clone(), record.metadata.offset + 1);
                        let msg = NatsPacket::Msg {
                            subject: subject.clone(),
                            sid: sid.clone(),
                            reply_to: None,
                            payload: record.data.clone(),
                        };
                        let wrapper = RobustMQPacketWrapper {
                            protocol: RobustMQProtocol::NATS,
                            extend: RobustMQWrapperExtend::NATS(NatsWrapperExtend {}),
                            packet: RobustMQPacket::NATS(msg),
                        };
                        if let Err(e) = connection_manager
                            .write_tcp_frame(connection_id, wrapper)
                            .await
                        {
                            error!(connection_id, "NATS subscribe push failed: {}", e);
                            return;
                        }
                    }
                }
                Err(_) => {
                    sleep(Duration::from_millis(1000)).await;
                }
            }
        }
    });

    // SUB has no immediate response
    None
}

pub fn process_unsub(_sid: &str, _max_msgs: Option<u32>) -> Option<NatsPacket> {
    None
}
