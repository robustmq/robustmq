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

use dashmap::DashMap;
use futures::SinkExt;
use protocol::mqtt::common::MqttPacket;
use protocol::mqtt::mqttv5::codec::Mqtt5Codec;
use std::time::Duration;
use tokio::sync::broadcast::Sender;
use tokio::time::sleep;
use tokio_util::codec::FramedWrite;
use tracing::error;

pub struct WriteStream {
    write_list:
        DashMap<String, FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, Mqtt5Codec>>,
    key: String,
    stop_sx: Sender<bool>,
}

impl WriteStream {
    pub fn new(stop_sx: Sender<bool>) -> Self {
        WriteStream {
            key: "default".to_string(),
            write_list: DashMap::with_capacity(2),
            stop_sx,
        }
    }

    pub fn add_write(
        &self,
        write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, Mqtt5Codec>,
    ) {
        self.write_list.insert(self.key.clone(), write);
    }

    pub async fn write_frame(&self, resp: MqttPacket) {
        loop {
            if let Ok(flag) = self.stop_sx.subscribe().try_recv() {
                if flag {
                    break;
                }
            }

            match self.write_list.try_get_mut(&self.key) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da.send(resp.clone()).await {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            error!(
                                "Resub Client Failed to write data to the response queue, error message: {:?}",
                                e
                            );
                        }
                    }
                }
                dashmap::try_result::TryResult::Absent => {
                    error!("Resub Client [write_frame]Connection management could not obtain an available connection.");
                }
                dashmap::try_result::TryResult::Locked => {
                    error!("Resub Client [write_frame]Connection management failed to get connection variable reference");
                }
            }
            sleep(Duration::from_secs(1)).await
        }
    }
}
