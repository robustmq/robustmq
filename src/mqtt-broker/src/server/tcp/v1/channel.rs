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

use crate::server::{
    connection::calc_child_channel_index,
    packet::{RequestPackage, ResponsePackage},
};
use dashmap::DashMap;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing::error;

#[derive(Clone)]
pub struct RequestChannel {
    pub request_send_channel: DashMap<usize, Sender<RequestPackage>>,
    pub handler_child_channels: DashMap<usize, Sender<RequestPackage>>,
    pub response_send_channel: DashMap<usize, Sender<ResponsePackage>>,
    pub response_child_channels: DashMap<usize, Sender<ResponsePackage>>,
    pub channel_size: usize,
}

impl RequestChannel {
    pub fn new(channel_size: usize) -> Self {
        RequestChannel {
            request_send_channel: DashMap::with_capacity(2),
            handler_child_channels: DashMap::with_capacity(2),
            response_send_channel: DashMap::with_capacity(2),
            response_child_channels: DashMap::with_capacity(2),
            channel_size,
        }
    }
    pub fn create_request_channel(&self) -> Receiver<RequestPackage> {
        let (sx, rx) = mpsc::channel::<RequestPackage>(self.channel_size);
        self.request_send_channel.insert(0, sx);
        rx
    }

    pub fn create_response_channel(&self) -> Receiver<ResponsePackage> {
        let (sx, rx) = mpsc::channel::<ResponsePackage>(self.channel_size);
        self.response_send_channel.insert(0, sx);
        rx
    }

    pub fn create_handler_child_channel(&self, index: usize) -> Receiver<RequestPackage> {
        let (sx, rx) = mpsc::channel::<RequestPackage>(1000);
        self.handler_child_channels.insert(index, sx);
        rx
    }

    pub fn create_response_child_channel(&self, index: usize) -> Receiver<ResponsePackage> {
        let (sx, rx) = mpsc::channel::<ResponsePackage>(1000);
        self.response_child_channels.insert(index, sx);
        rx
    }

    pub async fn send_response_channel(&self, response_package: ResponsePackage) {
        let sx = self.response_send_channel.get(&0).unwrap();
        if let Err(err) = sx.send(response_package).await {
            error!(
                "Failed to write data to the response queue, error message: {:?}",
                err
            );
        }
    }

    pub async fn send_request_channel(&self, request_package: RequestPackage) {
        let sx = self.request_send_channel.get(&0).unwrap();
        if let Err(err) = sx.send(request_package).await {
            error!(
                "Failed to write data to the request queue, error message: {:?}",
                err
            );
        }
    }

    pub fn get_avaiable_handler(
        &self,
        process_handler_seq: usize,
    ) -> Option<Sender<RequestPackage>> {
        let seq = calc_child_channel_index(process_handler_seq, self.handler_child_channels.len());
        if let Some(handler_sx) = self.handler_child_channels.get(&seq) {
            return Some(handler_sx.clone());
        }
        None
    }

    pub fn get_avaiable_response(
        &self,
        response_process_seq: usize,
    ) -> Option<Sender<ResponsePackage>> {
        let seq =
            calc_child_channel_index(response_process_seq, self.response_child_channels.len());
        if let Some(handler_sx) = self.response_child_channels.get(&seq) {
            return Some(handler_sx.clone());
        }
        None
    }
}
