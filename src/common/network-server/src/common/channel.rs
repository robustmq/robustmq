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

use crate::common::packet::{RequestPackage, ResponsePackage};
use dashmap::DashMap;
use metadata_struct::connection::{calc_child_channel_index, NetworkConnectionType};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing::error;

#[derive(Clone)]
pub struct RequestChannel {
    pub request_send_channel: DashMap<String, Sender<RequestPackage>>,
    pub handler_child_channels: DashMap<String, Sender<RequestPackage>>,
    pub response_send_channel: DashMap<String, Sender<ResponsePackage>>,
    pub response_child_channels: DashMap<String, Sender<ResponsePackage>>,
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
    pub fn create_request_channel(
        &self,
        network_type: &NetworkConnectionType,
    ) -> Receiver<RequestPackage> {
        let (sx, rx) = mpsc::channel::<RequestPackage>(self.channel_size);
        self.request_send_channel
            .insert(network_type.to_string(), sx);
        rx
    }

    pub fn create_response_channel(
        &self,
        network_type: &NetworkConnectionType,
    ) -> Receiver<ResponsePackage> {
        let (sx, rx) = mpsc::channel::<ResponsePackage>(self.channel_size);
        self.response_send_channel
            .insert(network_type.to_string(), sx);
        rx
    }

    pub fn create_handler_child_channel(
        &self,
        network_type: &NetworkConnectionType,
        index: usize,
    ) -> Receiver<RequestPackage> {
        let (sx, rx) = mpsc::channel::<RequestPackage>(self.channel_size);
        let key = self.key_name(network_type, index);
        self.handler_child_channels.insert(key, sx);
        rx
    }

    pub fn create_response_child_channel(
        &self,
        network_type: &NetworkConnectionType,
        index: usize,
    ) -> Receiver<ResponsePackage> {
        let (sx, rx) = mpsc::channel::<ResponsePackage>(self.channel_size);
        let key = self.key_name(network_type, index);
        self.response_child_channels.insert(key, sx);
        rx
    }

    pub async fn send_response_channel(
        &self,
        network_type: &NetworkConnectionType,
        response_package: ResponsePackage,
    ) {
        let sx = self.get_response_send_channel(network_type);
        if let Err(err) = sx.send(response_package).await {
            error!(
                "Failed to write data to the response queue, error message: {:?}",
                err
            );
        }
    }

    pub async fn send_request_channel(
        &self,
        network_type: &NetworkConnectionType,
        request_package: RequestPackage,
    ) {
        let sx = self.get_request_send_channel(network_type);
        if let Err(err) = sx.send(request_package).await {
            error!(
                "Failed to write data to the request queue, error message: {}",
                err.to_string()
            );
        }
    }

    pub fn get_request_send_channel(
        &self,
        network_type: &NetworkConnectionType,
    ) -> Sender<RequestPackage> {
        self.request_send_channel
            .get(&network_type.to_string())
            .unwrap()
            .clone()
    }

    pub fn get_response_send_channel(
        &self,
        network_type: &NetworkConnectionType,
    ) -> Sender<ResponsePackage> {
        self.response_send_channel
            .get(&network_type.to_string())
            .unwrap()
            .clone()
    }

    pub fn get_available_handler(
        &self,
        network_type: &NetworkConnectionType,
        process_handler_seq: usize,
    ) -> Option<Sender<RequestPackage>> {
        let index =
            calc_child_channel_index(process_handler_seq, self.handler_child_channels.len());
        let key = self.key_name(network_type, index);
        if let Some(handler_sx) = self.handler_child_channels.get(&key) {
            return Some(handler_sx.clone());
        }
        None
    }

    pub fn get_available_response(
        &self,
        network_type: &NetworkConnectionType,
        response_process_seq: usize,
    ) -> Option<Sender<ResponsePackage>> {
        let index =
            calc_child_channel_index(response_process_seq, self.response_child_channels.len());
        let key = self.key_name(network_type, index);
        if let Some(handler_sx) = self.response_child_channels.get(&key) {
            return Some(handler_sx.clone());
        }
        None
    }

    fn key_name(&self, network_type: &NetworkConnectionType, index: usize) -> String {
        format!("{network_type}_{index}")
    }
}
