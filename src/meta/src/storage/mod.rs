/*
 * Copyright (c) 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use serde::{Deserialize, Serialize};

const DB_COLUMN_FAMILY_META: &str = "meta";
const DB_COLUMN_FAMILY_CLUSTER: &str = "cluster";
const DB_COLUMN_FAMILY_MQTT: &str = "mqtt";

fn column_family_list() -> Vec<String> {
    let mut list = Vec::new();
    list.push(DB_COLUMN_FAMILY_META.to_string());
    list.push(DB_COLUMN_FAMILY_CLUSTER.to_string());
    list.push(DB_COLUMN_FAMILY_MQTT.to_string());
    return list;
}

#[derive(Deserialize, Serialize)]
pub enum StorageDataType {
    RegisterBroker,
    UnRegisterBroker,
}

#[derive(Deserialize, Serialize)]
pub struct StorageData {
    pub data_type: StorageDataType,
    pub data: Vec<u8>,
}

impl StorageData {
    pub fn new(data_type: StorageDataType, data: Vec<u8>) -> StorageData {
        return StorageData { data_type, data };
    }
}

pub mod data_struct;
pub mod raft_core;
pub mod raft_storage;
pub mod rocksdb;
