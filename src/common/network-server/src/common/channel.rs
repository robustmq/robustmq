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

use crate::common::packet::RequestPackage;
use tracing::error;

#[derive(Clone)]
pub struct RequestChannel {
    pub sender: async_channel::Sender<RequestPackage>,
    pub receiver: async_channel::Receiver<RequestPackage>,
    pub channel_size: usize,
}

impl RequestChannel {
    pub fn new(channel_size: usize) -> Self {
        let (sender, receiver) = async_channel::bounded(channel_size);
        RequestChannel {
            sender,
            receiver,
            channel_size,
        }
    }

    pub async fn send(&self, packet: RequestPackage) {
        if let Err(e) = self.sender.send(packet).await {
            error!("Failed to send packet to handler channel: {}", e);
        }
    }

    pub fn len(&self) -> usize {
        self.sender.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sender.is_empty()
    }
}
