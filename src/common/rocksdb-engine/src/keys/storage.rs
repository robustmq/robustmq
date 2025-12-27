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

use super::PREFIX_STORAGE;
use std::fmt::Write;

#[inline(always)]
pub fn shard_record_key(shard: &str, record_offset: u64) -> String {
    let mut key = String::with_capacity(32 + shard.len());
    key.push_str(PREFIX_STORAGE);
    key.push_str("record/");
    key.push_str(shard);
    key.push('/');
    let _ = write!(key, "{:020}", record_offset);
    key
}

#[inline(always)]
pub fn shard_record_key_prefix(shard: &str) -> String {
    let mut key = String::with_capacity(18 + shard.len());
    key.push_str(PREFIX_STORAGE);
    key.push_str("record/");
    key.push_str(shard);
    key.push('/');
    key
}

#[inline(always)]
pub fn earliest_offset_key(shard: &str) -> String {
    format!("{}offset/earliest/{}", PREFIX_STORAGE, shard)
}

#[inline(always)]
pub fn latest_offset_key(shard: &str) -> String {
    format!("{}offset/latest/{}", PREFIX_STORAGE, shard)
}

#[inline(always)]
pub fn key_index_key(shard: &str, record_key: &str) -> String {
    format!("{}index/key/{}/{}", PREFIX_STORAGE, shard, record_key)
}

#[inline(always)]
pub fn key_index_prefix(shard: &str) -> String {
    format!("{}index/key/{}/", PREFIX_STORAGE, shard)
}

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

#[inline(always)]
pub fn group_record_offsets_key(group: &str, shard: &str) -> String {
    format!("{}group/{}/{}", PREFIX_STORAGE, group, shard)
}

#[inline(always)]
pub fn group_record_offsets_key_prefix(group: &str) -> String {
    format!("{}group/{}/", PREFIX_STORAGE, group)
}

#[inline(always)]
pub fn shard_info_key(shard: &str) -> String {
    format!("{}shard/{}", PREFIX_STORAGE, shard)
}

#[inline(always)]
pub fn shard_info_key_prefix() -> String {
    format!("{}shard/", PREFIX_STORAGE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_key_formats() {
        let cases: [(_, &'static str); 15] = [
            (
                shard_record_key("shard1", 123),
                "/storage/record/shard1/00000000000000000123",
            ),
            (shard_record_key_prefix("shard1"), "/storage/record/shard1/"),
            (
                earliest_offset_key("shard1"),
                "/storage/offset/earliest/shard1",
            ),
            (latest_offset_key("shard1"), "/storage/offset/latest/shard1"),
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
                group_record_offsets_key("group1", "shard1"),
                "/storage/group/group1/shard1",
            ),
            (
                group_record_offsets_key_prefix("group1"),
                "/storage/group/group1/",
            ),
            (shard_info_key("shard1"), "/storage/shard/shard1"),
            (shard_info_key_prefix(), "/storage/shard/"),
            (
                timestamp_index_key("shard1", 1234567890, 100),
                "/storage/index/timestamp/shard1/00000000001234567890/00000000000000000100",
            ),
            (
                timestamp_index_prefix("shard1"),
                "/storage/index/timestamp/shard1/",
            ),
        ];

        for (actual, expected) in cases {
            assert_eq!(actual, expected);
        }
    }
}
