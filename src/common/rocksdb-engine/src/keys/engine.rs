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

use super::PREFIX_ENGINE;

// =====================================================================
// All storage-engine keys live under a single namespace, organised so
// that every key for a shard nests under `shard_prefix(shard)` and every
// per-segment key nests under `segment_prefix(shard, segment)`:
//
//   /engine/{shard}/
//       meta/{earliest,latest,high-watermark}
//       index/key/{key}                          (shard-level)
//       index/tag/{tag}/{offset}                 (shard-level)
//       index/timestamp/{ts}/{offset}            (shard-level)
//       segment/{segment}/
//           record/{offset}
//           position/{offset}
//           timestamp/{time}
//           leader-epoch/{epoch}
//
// Cleanup is therefore a single prefix delete: a whole shard via
// `shard_prefix`, one segment via `segment_prefix`. Adding a new key type
// under these prefixes needs no change to the delete paths.
// =====================================================================

// Roots.
#[inline]
pub fn shard_prefix(shard: &str) -> String {
    format!("{}{}/", PREFIX_ENGINE, shard)
}

#[inline]
pub fn segment_prefix(shard: &str, segment: u32) -> String {
    format!("{}{}/segment/{:010}/", PREFIX_ENGINE, shard, segment)
}

// Shard meta (offset markers).
#[inline]
pub fn shard_earliest_offset(shard: &str) -> String {
    format!("{}meta/earliest", shard_prefix(shard))
}

#[inline]
pub fn shard_latest_offset(shard: &str) -> String {
    format!("{}meta/latest", shard_prefix(shard))
}

#[inline]
pub fn shard_high_watermark_offset(shard: &str) -> String {
    format!("{}meta/high-watermark", shard_prefix(shard))
}

// Shard-level key index (record key -> offset; used for compaction).
// The record key is arbitrary binary (e.g. Kafka record keys), so this
// builds the key as raw bytes rather than a UTF-8 `String`.
#[inline]
pub fn key_index_key(shard: &str, record_key: &[u8]) -> Vec<u8> {
    let mut key = key_index_prefix(shard).into_bytes();
    key.extend_from_slice(record_key);
    key
}

#[inline]
pub fn key_index_prefix(shard: &str) -> String {
    format!("{}index/key/", shard_prefix(shard))
}

// Shard-level tag index (tag -> offsets).
#[inline]
pub fn tag_index_key(shard: &str, tag: &str, offset: u64) -> String {
    format!("{}index/tag/{}/{:020}", shard_prefix(shard), tag, offset)
}

#[inline]
pub fn tag_index_tag_prefix(shard: &str, tag: &str) -> String {
    format!("{}index/tag/{}/", shard_prefix(shard), tag)
}

#[inline]
pub fn tag_index_prefix(shard: &str) -> String {
    format!("{}index/tag/", shard_prefix(shard))
}

// Shard-level timestamp index (timestamp -> offsets).
#[inline]
pub fn timestamp_index_key(shard: &str, timestamp: u64, offset: u64) -> String {
    format!(
        "{}index/timestamp/{:020}/{:020}",
        shard_prefix(shard),
        timestamp,
        offset
    )
}

#[inline]
pub fn timestamp_index_prefix(shard: &str) -> String {
    format!("{}index/timestamp/", shard_prefix(shard))
}

// Segment-level record bytes (commitlog).
#[inline]
pub fn record_key(shard: &str, segment: u32, offset: u64) -> String {
    format!("{}record/{:020}", segment_prefix(shard, segment), offset)
}

#[inline]
pub fn record_prefix(shard: &str, segment: u32) -> String {
    format!("{}record/", segment_prefix(shard, segment))
}

// Segment-level position index (filesegment; offset -> file position).
#[inline]
pub fn position_index_key(shard: &str, segment: u32, offset: u64) -> String {
    format!("{}position/{:020}", segment_prefix(shard, segment), offset)
}

#[inline]
pub fn position_index_prefix(shard: &str, segment: u32) -> String {
    format!("{}position/", segment_prefix(shard, segment))
}

// Segment-level timestamp index (filesegment; time -> offset).
#[inline]
pub fn segment_timestamp_index_key(shard: &str, segment: u32, time_sec: u64) -> String {
    format!(
        "{}timestamp/{:020}",
        segment_prefix(shard, segment),
        time_sec
    )
}

#[inline]
pub fn segment_timestamp_index_prefix(shard: &str, segment: u32) -> String {
    format!("{}timestamp/", segment_prefix(shard, segment))
}

// Segment-level leader epoch history.
#[inline]
pub fn leader_epoch_key(shard: &str, segment: u32, epoch: u32) -> String {
    format!(
        "{}leader-epoch/{:010}",
        segment_prefix(shard, segment),
        epoch
    )
}

#[inline]
pub fn leader_epoch_prefix(shard: &str, segment: u32) -> String {
    format!("{}leader-epoch/", segment_prefix(shard, segment))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_key_formats() {
        let cases: [(_, &'static str); 16] = [
            (shard_prefix("s1"), "/engine/s1/"),
            (segment_prefix("s1", 3), "/engine/s1/segment/0000000003/"),
            (shard_earliest_offset("s1"), "/engine/s1/meta/earliest"),
            (shard_latest_offset("s1"), "/engine/s1/meta/latest"),
            (
                shard_high_watermark_offset("s1"),
                "/engine/s1/meta/high-watermark",
            ),
            (key_index_prefix("s1"), "/engine/s1/index/key/"),
            (
                tag_index_key("s1", "t1", 7),
                "/engine/s1/index/tag/t1/00000000000000000007",
            ),
            (tag_index_tag_prefix("s1", "t1"), "/engine/s1/index/tag/t1/"),
            (tag_index_prefix("s1"), "/engine/s1/index/tag/"),
            (
                timestamp_index_key("s1", 100, 7),
                "/engine/s1/index/timestamp/00000000000000000100/00000000000000000007",
            ),
            (timestamp_index_prefix("s1"), "/engine/s1/index/timestamp/"),
            (
                record_key("s1", 3, 7),
                "/engine/s1/segment/0000000003/record/00000000000000000007",
            ),
            (
                record_prefix("s1", 3),
                "/engine/s1/segment/0000000003/record/",
            ),
            (
                position_index_key("s1", 3, 7),
                "/engine/s1/segment/0000000003/position/00000000000000000007",
            ),
            (
                segment_timestamp_index_key("s1", 3, 100),
                "/engine/s1/segment/0000000003/timestamp/00000000000000000100",
            ),
            (
                leader_epoch_key("s1", 3, 5),
                "/engine/s1/segment/0000000003/leader-epoch/0000000005",
            ),
        ];

        for (actual, expected) in cases {
            assert_eq!(actual, expected);
        }

        assert_eq!(key_index_key("s1", b"k1"), b"/engine/s1/index/key/k1");
        assert_eq!(
            key_index_key("s1", &[0xff, 0x00, 0x01]),
            [b"/engine/s1/index/key/".as_slice(), &[0xff, 0x00, 0x01]].concat()
        );
    }
}
