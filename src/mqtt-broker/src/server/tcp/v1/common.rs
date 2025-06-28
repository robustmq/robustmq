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

use std::time::Duration;

use protocol::mqtt::common::{Error, MqttPacket};
use tokio::time::sleep;
use tracing::{debug, info};

use crate::{
    observability::metrics::packets::{record_received_error_metrics, record_received_metrics},
    server::{
        connection::{NetworkConnection, NetworkConnectionType},
        packet::RequestPackage,
        tcp::v1::channel::RequestChannel,
    },
};

pub async fn read_packet(
    package: Option<Result<MqttPacket, Error>>,
    request_channel: &RequestChannel,
    connection: &NetworkConnection,
    network_type: &NetworkConnectionType,
) {
    if let Some(pkg) = package {
        match pkg {
            Ok(pack) => {
                info!(
                    "recv {} packet:{:?}, connect_id:{}",
                    network_type, pack, connection.connection_id
                );
                record_received_metrics(connection, &pack, network_type);

                let package = RequestPackage::new(connection.connection_id, connection.addr, pack);
                request_channel
                    .send_request_channel(network_type, package.clone())
                    .await;
            }
            Err(e) => {
                record_received_error_metrics(network_type.clone());
                debug!(
                    "{} connection parsing packet format error message :{:?}",
                    network_type, e
                )
            }
        }
    } else {
        sleep(Duration::from_millis(1)).await;
    }
}
