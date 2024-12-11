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

use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageData {
    pub data_type: StorageDataType,
    pub value: Vec<u8>,
}

impl StorageData {
    pub fn new(data_type: StorageDataType, value: Vec<u8>) -> StorageData {
        StorageData { data_type, value }
    }
}

impl fmt::Display for StorageData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}, {:?})", self.data_type, self.value)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum StorageDataType {
    // Cluster
    ClusterAddNode,
    ClusterDeleteNode,
    ClusterAddCluster,
    ClusterDeleteCluster,
    ClusterSetResourceConfig,
    ClusterDeleteResourceConfig,
    ClusterSetIdempotentData,
    ClusterDeleteIdempotentData,

    ClusterSaveOffset,
    ClusterDeleteOffset,

    // Journal
    JournalSetShard,
    JournalDeleteShard,
    JournalSetSegment,
    JournalDeleteSegment,
    JournalSetSegmentMetadata,
    JournalDeleteSegmentMetadata,

    // kv
    KvSet,
    KvDelete,

    // mqtt
    MqttSetUser,
    MqttDeleteUser,
    MqttSetTopic,
    MqttDeleteTopic,
    MqttSetSession,
    MqttDeleteSession,
    MqttUpdateSession,
    MqttSaveLastWillMessage,
    MqttSetAcl,
    MqttDeleteAcl,
    MqttSetBlacklist,
    MqttDeleteBlacklist,
    MqttSetNxExclusiveTopic,
    MqttDeleteExclusiveTopic,
}
