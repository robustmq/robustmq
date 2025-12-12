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
/// Format: /record/{shard}/record/{offset:020}
#[inline(always)]
pub fn shard_record_key(shard: &str, record_offset: u64) -> String {
    let mut key = String::with_capacity(17 + shard.len() + 20);
    key.push_str("/record/");
    key.push_str(shard);
    key.push_str("/record/");
    let _ = write!(key, "{:020}", record_offset);
    key
}

/// Generate record key prefix for range queries
/// Format: /record/{shard}/record/
#[inline(always)]
pub fn shard_record_key_prefix(shard: &str) -> String {
    let mut key = String::with_capacity(17 + shard.len());
    key.push_str("/record/");
    key.push_str(shard);
    key.push_str("/record/");
    key
}

/// Generate shard offset key to store the next offset
/// Format: /offset/{shard}
#[inline(always)]
pub fn shard_offset_key(shard: &str) -> String {
    let mut key = String::with_capacity(9 + shard.len());
    key.push_str("/offset/");
    key.push_str(shard);
    key
}

/// Generate key-to-offset mapping key
/// Format: /key/{shard}/{record_key}
#[inline(always)]
pub fn key_offset_key(shard: &str, record_key: &str) -> String {
    let mut key = String::with_capacity(7 + shard.len() + record_key.len());
    key.push_str("/key/");
    key.push_str(shard);
    key.push('/');
    key.push_str(record_key);
    key
}

/// Generate tag-to-offset mapping key for a specific offset
/// Format: /tag/{shard}/{tag}/{offset:020}
#[inline(always)]
pub fn tag_offsets_key(shard: &str, tag: &str, offset: u64) -> String {
    let mut key = String::with_capacity(7 + shard.len() + tag.len() + 20);
    key.push_str("/tag/");
    key.push_str(shard);
    key.push('/');
    key.push_str(tag);
    key.push('/');
    let _ = write!(key, "{:020}", offset);
    key
}

/// Generate tag prefix for range queries
/// Format: /tag/{shard}/{tag}/
#[inline(always)]
pub fn tag_offsets_key_prefix(shard: &str, tag: &str) -> String {
    let mut key = String::with_capacity(7 + shard.len() + tag.len());
    key.push_str("/tag/");
    key.push_str(shard);
    key.push('/');
    key.push_str(tag);
    key.push('/');
    key
}

/// Generate consumer group offset key
/// Format: /group/{group}/{shard}
#[inline(always)]
pub fn group_record_offsets_key(group: &str, shard: &str) -> String {
    let mut key = String::with_capacity(9 + group.len() + shard.len());
    key.push_str("/group/");
    key.push_str(group);
    key.push('/');
    key.push_str(shard);
    key
}

/// Generate consumer group prefix for range queries
/// Format: /group/{group}/
#[inline(always)]
pub fn group_record_offsets_key_prefix(group: &str) -> String {
    let mut key = String::with_capacity(8 + group.len());
    key.push_str("/group/");
    key.push_str(group);
    key.push('/');
    key
}

/// Generate shard info key to store ShardInfo metadata
/// Format: /shard/{shard}
#[inline(always)]
pub fn shard_info_key(shard: &str) -> String {
    let mut key = String::with_capacity(8 + shard.len());
    key.push_str("/shard/");
    key.push_str(shard);
    key
}

/// Generate timestamp-to-offset mapping key
/// Format: /timestamp/{shard}/{timestamp:020}/{offset:020}
#[inline(always)]
pub fn timestamp_offset_key(shard: &str, timestamp: u64, offset: u64) -> String {
    let mut key = String::with_capacity(13 + shard.len() + 40);
    key.push_str("/timestamp/");
    key.push_str(shard);
    key.push('/');
    let _ = write!(key, "{:020}/{:020}", timestamp, offset);
    key
}

/// Generate timestamp index prefix for range queries
/// Format: /timestamp/{shard}/
#[inline(always)]
pub fn timestamp_offset_key_prefix(shard: &str) -> String {
    let mut key = String::with_capacity(13 + shard.len());
    key.push_str("/timestamp/");
    key.push_str(shard);
    key.push('/');
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_record_key() {
        let key = shard_record_key("shard1", 123);
        assert_eq!(key, "/record/shard1/record/00000000000000000123");
    }

    #[test]
    fn test_shard_record_key_prefix() {
        let key = shard_record_key_prefix("shard1");
        assert_eq!(key, "/record/shard1/record/");
    }

    #[test]
    fn test_shard_offset_key() {
        let key = shard_offset_key("shard1");
        assert_eq!(key, "/offset/shard1");
    }

    #[test]
    fn test_key_offset_key() {
        let key = key_offset_key("shard1", "mykey");
        assert_eq!(key, "/key/shard1/mykey");
    }

    #[test]
    fn test_tag_offsets_key() {
        let key = tag_offsets_key("shard1", "tag1", 456);
        assert_eq!(key, "/tag/shard1/tag1/00000000000000000456");
    }

    #[test]
    fn test_tag_offsets_key_prefix() {
        let key = tag_offsets_key_prefix("shard1", "tag1");
        assert_eq!(key, "/tag/shard1/tag1/");
    }

    #[test]
    fn test_group_record_offsets_key() {
        let key = group_record_offsets_key("group1", "shard1");
        assert_eq!(key, "/group/group1/shard1");
    }

    #[test]
    fn test_group_record_offsets_key_prefix() {
        let key = group_record_offsets_key_prefix("group1");
        assert_eq!(key, "/group/group1/");
    }

    #[test]
    fn test_shard_info_key() {
        let key = shard_info_key("shard1");
        assert_eq!(key, "/shard/shard1");
    }

    #[test]
    fn test_timestamp_offset_key() {
        let key = timestamp_offset_key("shard1", 1234567890, 100);
        assert_eq!(
            key,
            "/timestamp/shard1/00000000001234567890/00000000000000000100"
        );
    }

    #[test]
    fn test_timestamp_offset_key_prefix() {
        let key = timestamp_offset_key_prefix("shard1");
        assert_eq!(key, "/timestamp/shard1/");
    }
}
