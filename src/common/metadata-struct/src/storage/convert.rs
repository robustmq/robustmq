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

use common_base::tools::now_second;

use crate::adapter::adapter_record::{AdapterWriteRecord, RecordHeader};
use crate::storage::storage_record::{
    Header as StorageHeader, StorageRecord, StorageRecordMetadata,
};

pub fn convert_adapter_headers_to_storage(
    headers: Option<Vec<RecordHeader>>,
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
    record: AdapterWriteRecord,
    shard: &str,
    offset: u64,
) -> StorageRecord {
    let metadata = StorageRecordMetadata::build(offset, shard.to_string(), 0)
        .with_header(convert_adapter_headers_to_storage(record.header))
        .with_key(record.key)
        .with_tags(record.tags)
        .with_timestamp(now_second())
        .with_crc_from_data(&record.data);

    StorageRecord {
        metadata,
        data: record.data,
    }
}

pub fn convert_engine_record_to_adapter(record: StorageRecord) -> AdapterWriteRecord {
    let mut result = AdapterWriteRecord::new(
        record.metadata.shard.clone(),
        record.data,
    );

    if let Some(key) = record.metadata.key {
        result = result.with_key(key);
    }

    if let Some(tags) = record.metadata.tags {
        result = result.with_tags(tags);
    }

    if let Some(hs) = record.metadata.header {
        let headers = hs
            .into_iter()
            .map(|h| RecordHeader {
                name: h.name,
                value: h.value,
            })
            .collect();
        result = result.with_header(headers);
    }

    result
}
