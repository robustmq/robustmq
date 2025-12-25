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

use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StorageData {
    pub data_type: StorageDataType,
    pub value: Bytes,
}

impl StorageData {
    pub fn new(data_type: StorageDataType, value: Bytes) -> StorageData {
        StorageData { data_type, value }
    }
}

impl fmt::Display for StorageData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}, {:?})", self.data_type, self.value)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum StorageDataType {
    // Cluster
    ClusterAddNode,
    ClusterDeleteNode,

    // KV
    KvSet,
    KvDelete,

    // Common
    SchemaSet,
    SchemaDelete,
    SchemaBindSet,
    SchemaBindDelete,
    ResourceConfigSet,
    ResourceConfigDelete,
    OffsetSet,
    OffsetDelete,

    // StorageEngine
    StorageEngineSetShard,
    StorageEngineDeleteShard,
    StorageEngineSetSegment,
    StorageEngineDeleteSegment,
    StorageEngineSetSegmentMetadata,
    StorageEngineDeleteSegmentMetadata,

    // mqtt
    MqttSetUser,
    MqttDeleteUser,
    MqttSetTopic,
    MqttDeleteTopic,
    MqttSetRetainMessage,
    MqttDeleteRetainMessage,
    MqttSetSession,
    MqttDeleteSession,
    MqttUpdateSession,
    MqttSaveLastWillMessage,
    MqttSetAcl,
    MqttDeleteAcl,
    MqttSetBlacklist,
    MqttDeleteBlacklist,
    MqttCreateTopicRewriteRule,
    MqttDeleteTopicRewriteRule,
    MqttSetSubscribe,
    MqttDeleteSubscribe,
    MqttSetConnector,
    MqttDeleteConnector,
    MqttSetAutoSubscribeRule,
    MqttDeleteAutoSubscribeRule,
}

impl fmt::Display for StorageDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageDataType::ClusterAddNode => write!(f, "ClusterAddNode"),
            StorageDataType::ClusterDeleteNode => write!(f, "ClusterDeleteNode"),

            StorageDataType::KvSet => write!(f, "KvSet"),
            StorageDataType::KvDelete => write!(f, "KvDelete"),

            StorageDataType::SchemaSet => write!(f, "SchemaSet"),
            StorageDataType::SchemaDelete => write!(f, "SchemaDelete"),
            StorageDataType::SchemaBindSet => write!(f, "SchemaBindSet"),
            StorageDataType::SchemaBindDelete => write!(f, "SchemaBindDelete"),
            StorageDataType::ResourceConfigSet => write!(f, "ResourceConfigSet"),
            StorageDataType::ResourceConfigDelete => write!(f, "ResourceConfigDelete"),
            StorageDataType::OffsetSet => write!(f, "OffsetSet"),
            StorageDataType::OffsetDelete => write!(f, "OffsetDelete"),

            StorageDataType::StorageEngineSetShard => write!(f, "StorageEngineSetShard"),
            StorageDataType::StorageEngineDeleteShard => write!(f, "StorageEngineDeleteShard"),
            StorageDataType::StorageEngineSetSegment => write!(f, "StorageEngineSetSegment"),
            StorageDataType::StorageEngineDeleteSegment => write!(f, "StorageEngineDeleteSegment"),
            StorageDataType::StorageEngineSetSegmentMetadata => {
                write!(f, "StorageEngineSetSegmentMetadata")
            }
            StorageDataType::StorageEngineDeleteSegmentMetadata => {
                write!(f, "StorageEngineDeleteSegmentMetadata")
            }

            StorageDataType::MqttSetUser => write!(f, "MqttSetUser"),
            StorageDataType::MqttDeleteUser => write!(f, "MqttDeleteUser"),
            StorageDataType::MqttSetTopic => write!(f, "MqttSetTopic"),
            StorageDataType::MqttDeleteTopic => write!(f, "MqttDeleteTopic"),
            StorageDataType::MqttSetRetainMessage => write!(f, "MqttSetRetainMessage"),
            StorageDataType::MqttDeleteRetainMessage => write!(f, "MqttDeleteRetainMessage"),
            StorageDataType::MqttSetSession => write!(f, "MqttSetSession"),
            StorageDataType::MqttDeleteSession => write!(f, "MqttDeleteSession"),
            StorageDataType::MqttUpdateSession => write!(f, "MqttUpdateSession"),
            StorageDataType::MqttSaveLastWillMessage => write!(f, "MqttSaveLastWillMessage"),
            StorageDataType::MqttSetAcl => write!(f, "MqttSetAcl"),
            StorageDataType::MqttDeleteAcl => write!(f, "MqttDeleteAcl"),
            StorageDataType::MqttSetBlacklist => write!(f, "MqttSetBlacklist"),
            StorageDataType::MqttDeleteBlacklist => write!(f, "MqttDeleteBlacklist"),
            StorageDataType::MqttCreateTopicRewriteRule => write!(f, "MqttCreateTopicRewriteRule"),
            StorageDataType::MqttDeleteTopicRewriteRule => write!(f, "MqttDeleteTopicRewriteRule"),
            StorageDataType::MqttSetSubscribe => write!(f, "MqttSetSubscribe"),
            StorageDataType::MqttDeleteSubscribe => write!(f, "MqttDeleteSubscribe"),
            StorageDataType::MqttSetConnector => write!(f, "MqttSetConnector"),
            StorageDataType::MqttDeleteConnector => write!(f, "MqttDeleteConnector"),
            StorageDataType::MqttSetAutoSubscribeRule => write!(f, "MqttSetAutoSubscribeRule"),
            StorageDataType::MqttDeleteAutoSubscribeRule => {
                write!(f, "MqttDeleteAutoSubscribeRule")
            }
        }
    }
}
