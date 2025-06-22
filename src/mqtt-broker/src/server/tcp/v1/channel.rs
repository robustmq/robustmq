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

use crate::server::packet::{RequestPackage, ResponsePackage};
use dashmap::DashMap;
use tokio::sync::mpsc::Sender;

pub struct RequestChannel {
    request_channel: Sender<RequestPackage>,
    handler_child_channels: DashMap<usize, Sender<RequestPackage>>,
    response_channel: Sender<ResponsePackage>,
    response_child_channels: DashMap<usize, Sender<RequestPackage>>,
}

impl RequestChannel {
    pub fn new(
        request_channel: Sender<RequestPackage>,
        response_channel: Sender<ResponsePackage>,
    ) -> Self {
        RequestChannel {
            request_channel,
            handler_child_channels: DashMap::with_capacity(2),
            response_channel,
            response_child_channels: DashMap::with_capacity(2),
        }
    }
}
