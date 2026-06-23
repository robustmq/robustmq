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

use super::{PREFIX_ENGINE, PREFIX_STORAGE};

// =====================================================================
// Engine namespace (PREFIX_ENGINE = "/engine/") — file-segment engine:
// per-segment metadata, position/timestamp indexes and leader epochs.
// =====================================================================

// Base prefixes.
#[inline]
fn shard_base(shard: &str) -> String {
    format!("{}{}/", PREFIX_ENGINE, shard)
}

#[inline]
pub fn segment_base(shard: &str, segment: u32) -> String {
    format!("{}segment/{}/{:010}/", PREFIX_ENGINE, shard, segment)
}

// Shard offset markers (earliest / high-watermark / latest).
pub fn shard_earliest_offset(shard: &str) -> String {
    format!("{}earliest/offset", shard_base(shard))
}

pub fn shard_high_watermark_offset(shard: &str) -> String {
    format!("{}high_watermark/offset", shard_base(shard))
}

pub fn shard_latest_offset(shard: &str) -> String {
    format!("{}latest/offset", shard_base(shard))
}

// Segment position index (offset -> file position).
pub fn index_position_key(shard: &str, segment: u32, offset: u64) -> String {
    format!("{}position/{:020}", segment_base(shard, segment), offset)
}

pub fn index_position_key_prefix(shard: &str, segment: u32) -> String {
    format!("{}position/", segment_base(shard, segment))
}

// Segment timestamp index (time -> offset).
pub fn index_timestamp_key(shard: &str, segment: u32, time_sec: u64) -> String {
    format!("{}timestamp/{:020}", segment_base(shard, segment), time_sec)
}

pub fn index_timestamp_key_prefix(shard: &str, segment: u32) -> String {
    format!("{}timestamp/", segment_base(shard, segment))
}

// Leader epoch history (per segment).
pub fn leader_epoch_entry_key(shard: &str, segment: u32, epoch: u32) -> String {
    format!("{}leader-epoch/{:010}", segment_base(shard, segment), epoch)
}

pub fn leader_epoch_prefix(shard: &str, segment: u32) -> String {
    format!("{}leader-epoch/", segment_base(shard, segment))
}

// Shard-level tag / key indexes.
pub fn index_tag_key(shard: &str, tag: String, offset: u64) -> String {
    format!("{}tag/{}/{:020}", shard_base(shard), tag, offset)
}

pub fn index_tag_key_prefix(shard: &str, tag: &str) -> String {
    format!("{}tag/{}/", shard_base(shard), tag)
}

pub fn index_key_key(shard: &str, key: String) -> String {
    format!("{}key/{}", shard_base(shard), key)
}

// =====================================================================
// Storage namespace (PREFIX_STORAGE = "/storage/") — commitlog records
// (memory / rocksdb): record bytes, segment LEO and key/tag/timestamp
// indexes.
// =====================================================================

// Record bytes.
#[inline(always)]
pub fn shard_record_key(shard: &str, segment_seq: u32, record_offset: u64) -> String {
    format!(
        "{}record/{}/{:010}/{:020}",
        PREFIX_STORAGE, shard, segment_seq, record_offset
    )
}

#[inline(always)]
pub fn shard_record_key_prefix(shard: &str, segment_seq: u32) -> String {
    format!("{}record/{}/{:010}/", PREFIX_STORAGE, shard, segment_seq)
}

#[inline(always)]
pub fn shard_record_shard_prefix(shard: &str) -> String {
    format!("{}record/{}/", PREFIX_STORAGE, shard)
}

// Segment LEO (log end offset).
#[inline(always)]
pub fn shard_segment_leo_key(shard: &str, segment_seq: u32) -> String {
    format!("{}record-leo/{}/{:010}", PREFIX_STORAGE, shard, segment_seq)
}

#[inline(always)]
pub fn shard_segment_leo_shard_prefix(shard: &str) -> String {
    format!("{}record-leo/{}/", PREFIX_STORAGE, shard)
}

// Key index (record key -> offset).
#[inline(always)]
pub fn key_index_key(shard: &str, record_key: &str) -> String {
    format!("{}index/key/{}/{}", PREFIX_STORAGE, shard, record_key)
}

#[inline(always)]
pub fn key_index_prefix(shard: &str) -> String {
    format!("{}index/key/{}/", PREFIX_STORAGE, shard)
}

// Tag index (tag -> offsets).
#[inline(always)]
pub fn tag_index_key(shard: &str, tag: &str, offset: u64) -> String {
    format!(
        "{}index/tag/{}/{}/{:020}",
        PREFIX_STORAGE, shard, tag, offset
    )
}

#[inline(always)]
pub fn tag_index_prefix(shard: &str) -> String {
    format!("{}index/tag/{}/", PREFIX_STORAGE, shard)
}

#[inline(always)]
pub fn tag_index_tag_prefix(shard: &str, tag: &str) -> String {
    format!("{}index/tag/{}/{}/", PREFIX_STORAGE, shard, tag)
}

// Timestamp index (timestamp -> offsets).
#[inline(always)]
pub fn timestamp_index_key(shard: &str, timestamp: u64, offset: u64) -> String {
    format!(
        "{}index/timestamp/{}/{:020}/{:020}",
        PREFIX_STORAGE, shard, timestamp, offset
    )
}

#[inline(always)]
pub fn timestamp_index_prefix(shard: &str) -> String {
    format!("{}index/timestamp/{}/", PREFIX_STORAGE, shard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_key_formats() {
        let cases: [(_, &'static str); 11] = [
            (
                shard_record_key("shard1", 0, 123),
                "/storage/record/shard1/0000000000/00000000000000000123",
            ),
            (
                shard_record_key_prefix("shard1", 0),
                "/storage/record/shard1/0000000000/",
            ),
            (
                shard_record_shard_prefix("shard1"),
                "/storage/record/shard1/",
            ),
            (
                shard_segment_leo_key("shard1", 0),
                "/storage/record-leo/shard1/0000000000",
            ),
            (
                shard_segment_leo_shard_prefix("shard1"),
                "/storage/record-leo/shard1/",
            ),
            (
                key_index_key("shard1", "mykey"),
                "/storage/index/key/shard1/mykey",
            ),
            (key_index_prefix("shard1"), "/storage/index/key/shard1/"),
            (
                tag_index_key("shard1", "tag1", 456),
                "/storage/index/tag/shard1/tag1/00000000000000000456",
            ),
            (
                tag_index_tag_prefix("shard1", "tag1"),
                "/storage/index/tag/shard1/tag1/",
            ),
            (tag_index_prefix("shard1"), "/storage/index/tag/shard1/"),
            (
                timestamp_index_key("shard1", 1234567890, 100),
                "/storage/index/timestamp/shard1/00000000001234567890/00000000000000000100",
            ),
        ];

        for (actual, expected) in cases {
            assert_eq!(actual, expected);
        }
    }
}
