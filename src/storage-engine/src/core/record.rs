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

use bytes::Bytes;
use common_base::tools::now_second;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug)]
pub struct StorageEngineRecordMetadata {
    pub offset: u64,
    pub shard: String,
    pub segment: u32,
    pub key: Option<String>,
    pub tags: Option<Vec<String>>,
    pub create_t: u64,
}

impl StorageEngineRecordMetadata {
    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<_, 256>(self).unwrap().to_vec()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let archived = rkyv::archived_root::<Self>(bytes);
            let deserialized: Self = archived.deserialize(&mut rkyv::Infallible).unwrap();
            Ok(deserialized)
        }
    }

    pub fn new(
        offset: u64,
        shard: &str,
        segment: u32,
        key: &Option<String>,
        tags: &Option<Vec<String>>,
    ) -> Self {
        StorageEngineRecordMetadata {
            offset,
            shard: shard.to_string(),
            segment,
            key: key.clone(),
            tags: tags.clone(),
            create_t: now_second(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StorageEngineRecord {
    pub metadata: StorageEngineRecordMetadata,
    pub data: Bytes,
}

impl StorageEngineRecord {
    pub fn len(&self) -> u64 {
        self.data.len() as u64
    }
}
