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

use std::fmt::Write;

/// Generate record key for a specific offset
/// Format: /sm/r/{shard}/{offset:020}
#[inline(always)]
pub fn shard_record_key(shard: &str, record_offset: u64) -> String {
    let mut key = String::with_capacity(32 + shard.len());
    key.push_str("/sm/r/");
    key.push_str(shard);
    key.push_str("/");
    let _ = write!(key, "{:020}", record_offset);
    key
}

/// Generate record key prefix for range queries
/// Format: /sm/r/{shard}/
#[inline(always)]
pub fn shard_record_key_prefix(shard: &str) -> String {
    let mut key = String::with_capacity(18 + shard.len());
    key.push_str("/sm/r/");
    key.push_str(shard);
    key.push_str("/");
    key
}

/// Generate shard earliest offset key
/// Format: /sm/o/e/{shard}
#[inline(always)]
pub fn earliest_offset_key(shard: &str) -> String {
    let mut key = String::with_capacity(20 + shard.len());
    key.push_str("/sm/o/e/");
    key.push_str(shard);
    key
}

/// Generate shard latest offset key to store the next offset
/// Format: /sm/o/l/{shard}
#[inline(always)]
pub fn latest_offset_key(shard: &str) -> String {
    let mut key = String::with_capacity(20 + shard.len());
    key.push_str("/sm/o/l/");
    key.push_str(shard);
    key
}

/// Generate key index key (key -> offset info)
/// Format: /sm/i/k/{shard}/{record_key}
#[inline(always)]
pub fn key_index_key(shard: &str, record_key: &str) -> String {
    let mut key = String::with_capacity(26 + shard.len() + record_key.len());
    key.push_str("/sm/i/k/");
    key.push_str(shard);
    key.push('/');
    key.push_str(record_key);
    key
}

/// Generate key index prefix for a shard
/// Format: /sm/i/k/{shard}/
#[inline(always)]
pub fn key_index_prefix(shard: &str) -> String {
    let mut key = String::with_capacity(14 + shard.len());
    key.push_str("/sm/i/k/");
    key.push_str(shard);
    key.push('/');
    key
}

/// Generate tag index key for a specific offset
/// Format: /sm/i/t/{shard}/{tag}/{offset:020}
#[inline(always)]
pub fn tag_index_key(shard: &str, tag: &str, offset: u64) -> String {
    let mut key = String::with_capacity(34 + shard.len() + tag.len() + 20);
    key.push_str("/sm/i/t/");
    key.push_str(shard);
    key.push('/');
    key.push_str(tag);
    key.push('/');
    let _ = write!(key, "{:020}", offset);
    key
}

/// Generate tag index prefix for a shard (all tags)
/// Format: /sm/i/t/{shard}/
#[inline(always)]
pub fn tag_index_prefix(shard: &str) -> String {
    let mut key = String::with_capacity(12 + shard.len());
    key.push_str("/sm/i/t/");
    key.push_str(shard);
    key.push('/');
    key
}

/// Generate tag index prefix for a specific tag
/// Format: /sm/i/t/{shard}/{tag}/
#[inline(always)]
pub fn tag_index_tag_prefix(shard: &str, tag: &str) -> String {
    let mut key = String::with_capacity(26 + shard.len() + tag.len());
    key.push_str("/sm/i/t/");
    key.push_str(shard);
    key.push('/');
    key.push_str(tag);
    key.push('/');
    key
}

/// Generate timestamp index key (timestamp -> offset info)
/// Format: /sm/i/ts/{shard}/{timestamp:020}/{offset:020}
#[inline(always)]
pub fn timestamp_index_key(shard: &str, timestamp: u64, offset: u64) -> String {
    let mut key = String::with_capacity(60 + shard.len());
    key.push_str("/sm/i/ts/");
    key.push_str(shard);
    key.push('/');
    let _ = write!(key, "{:020}/{:020}", timestamp, offset);
    key
}

/// Generate timestamp index prefix for range queries
/// Format: /sm/i/ts/{shard}/
#[inline(always)]
pub fn timestamp_index_prefix(shard: &str) -> String {
    let mut key = String::with_capacity(24 + shard.len());
    key.push_str("/sm/i/ts/");
    key.push_str(shard);
    key.push('/');
    key
}

/// Generate consumer group offset key
/// Format: /sm/g/{group}/{shard}
#[inline(always)]
pub fn group_record_offsets_key(group: &str, shard: &str) -> String {
    let mut key = String::with_capacity(18 + group.len() + shard.len());
    key.push_str("/sm/g/");
    key.push_str(group);
    key.push('/');
    key.push_str(shard);
    key
}

/// Generate consumer group prefix for range queries
/// Format: /sm/g/{group}/
#[inline(always)]
pub fn group_record_offsets_key_prefix(group: &str) -> String {
    let mut key = String::with_capacity(14 + group.len());
    key.push_str("/sm/g/");
    key.push_str(group);
    key.push('/');
    key
}

/// Generate shard info key to store ShardInfo metadata
/// Format: /sm/s/{shard}
#[inline(always)]
pub fn shard_info_key(shard: &str) -> String {
    let mut key = String::with_capacity(14 + shard.len());
    key.push_str("/sm/s/");
    key.push_str(shard);
    key
}

/// Generate shard info key prefix for listing shards
/// Format: /sm/s/
#[inline(always)]
pub fn shard_info_key_prefix() -> String {
    let mut key = String::with_capacity(6);
    key.push_str("/sm/s/");
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_key_formats() {
        let cases: [(_, &'static str); 15] = [
            (
                shard_record_key("shard1", 123),
                "/sm/r/shard1/00000000000000000123",
            ),
            (
                shard_record_key_prefix("shard1"),
                "/sm/r/shard1/",
            ),
            (
                earliest_offset_key("shard1"),
                "/sm/o/e/shard1",
            ),
            (
                latest_offset_key("shard1"),
                "/sm/o/l/shard1",
            ),
            (
                key_index_key("shard1", "mykey"),
                "/sm/i/k/shard1/mykey",
            ),
            (
                key_index_prefix("shard1"),
                "/sm/i/k/shard1/",
            ),
            (
                tag_index_key("shard1", "tag1", 456),
                "/sm/i/t/shard1/tag1/00000000000000000456",
            ),
            (
                tag_index_tag_prefix("shard1", "tag1"),
                "/sm/i/t/shard1/tag1/",
            ),
            (
                tag_index_prefix("shard1"),
                "/sm/i/t/shard1/",
            ),
            (
                group_record_offsets_key("group1", "shard1"),
                "/sm/g/group1/shard1",
            ),
            (
                group_record_offsets_key_prefix("group1"),
                "/sm/g/group1/",
            ),
            (
                shard_info_key("shard1"),
                "/sm/s/shard1",
            ),
            (
                shard_info_key_prefix(),
                "/sm/s/",
            ),
            (
                timestamp_index_key("shard1", 1234567890, 100),
                "/sm/i/ts/shard1/00000000001234567890/00000000000000000100",
            ),
            (
                timestamp_index_prefix("shard1"),
                "/sm/i/ts/shard1/",
            ),
        ];

        for (actual, expected) in cases {
            assert_eq!(actual, expected);
    }
}
}

