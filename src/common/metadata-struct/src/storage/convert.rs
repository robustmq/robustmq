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

use crate::adapter::record::{Header as AdapterHeader, StorageAdapterRecord};
use crate::storage::record::{
    Header as StorageHeader, StorageEngineRecord, StorageEngineRecordMetadata,
};

pub fn convert_adapter_headers_to_storage(
    headers: Option<Vec<AdapterHeader>>,
) -> Option<Vec<StorageHeader>> {
    headers.map(|hs| {
        hs.into_iter()
            .map(|h| StorageHeader {
                name: h.name,
                value: h.value,
            })
            .collect()
    })
}

pub fn convert_adapter_record_to_engine(
    record: StorageAdapterRecord,
    shard: &str,
    offset: u64,
) -> StorageEngineRecord {
    let metadata = StorageEngineRecordMetadata::build(offset, shard.to_string(), 0)
        .with_header(convert_adapter_headers_to_storage(record.header))
        .with_key(record.key)
        .with_tags(record.tags)
        .with_timestamp(record.timestamp)
        .with_crc_from_data(&record.data);

    StorageEngineRecord {
        metadata,
        data: record.data,
    }
}

pub fn convert_engine_record_to_adapter(record: StorageEngineRecord) -> StorageAdapterRecord {
    let header = record.metadata.header.map(|hs| {
        hs.into_iter()
            .map(|h| AdapterHeader {
                name: h.name,
                value: h.value,
            })
            .collect()
    });

    StorageAdapterRecord {
        pkid: record.metadata.offset,
        header,
        key: record.metadata.key,
        data: record.data,
        tags: record.metadata.tags,
        timestamp: record.metadata.create_t,
    }
}
