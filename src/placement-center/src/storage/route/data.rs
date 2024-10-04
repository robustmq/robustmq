use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageData {
    pub data_type: StorageDataType,
    pub value: Vec<u8>,
}

impl StorageData {
    pub fn new(data_type: StorageDataType, value: Vec<u8>) -> StorageData {
        return StorageData {
            data_type,
            value: value,
        };
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
    ClusterRegisterNode,
    ClusterUngisterNode,
    ClusterSetResourceConfig,
    ClusterDeleteResourceConfig,
    ClusterSetIdempotentData,
    ClusterDeleteIdempotentData,

    // Journal
    JournalCreateShard,
    JournalDeleteShard,
    JournalCreateSegment,
    JournalDeleteSegment,

    // kv
    KvSet,
    KvDelete,

    // mqtt
    MQTTCreateUser,
    MQTTDeleteUser,
    MQTTCreateTopic,
    MQTTDeleteTopic,
    MQTTSetTopicRetainMessage,
    MQTTCreateSession,
    MQTTDeleteSession,
    MQTTUpdateSession,
    MQTTSaveLastWillMessage,
    MQTTCreateAcl,
    MQTTDeleteAcl,
    MQTTCreateBlacklist,
    MQTTDeleteBlacklist,
}
