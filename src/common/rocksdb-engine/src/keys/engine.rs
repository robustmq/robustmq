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

#[inline]
fn segment_base_by_shard(shard: &str) -> String {
    format!("{}/{}/", PREFIX_ENGINE, shard)
}

#[inline]
pub fn segment_base_by_segment(shard: &str, segment: u32) -> String {
    format!("{}segment/{}/{}/", PREFIX_ENGINE, shard, segment)
}

// shard cursor offset
pub fn offset_segment_cursor_offset(shard: &str) -> String {
    format!("{}cursor/offset", segment_base_by_shard(shard))
}

// Segment start/end  offset/timestamp
pub fn offset_segment_start(shard: &str, segment: u32) -> String {
    format!("{}offset/start", segment_base_by_segment(shard, segment))
}

pub fn offset_segment_end(shard: &str, segment: u32) -> String {
    format!("{}offset/end", segment_base_by_segment(shard, segment))
}

pub fn timestamp_segment_start(shard: &str, segment: u32) -> String {
    format!("{}timestamp/start", segment_base_by_segment(shard, segment))
}

pub fn timestamp_segment_end(shard: &str, segment: u32) -> String {
    format!("{}timestamp/end", segment_base_by_segment(shard, segment))
}

// index(position/timestamp/tag/key)
pub fn index_position_key(shard: &str, offset: u64) -> String {
    format!("{}position/{}", segment_base_by_shard(shard), offset)
}

pub fn index_position_key_prefix(shard: &str) -> String {
    format!("{}/position/", segment_base_by_shard(shard))
}

pub fn index_timestamp_key(shard: &str, time_sec: u64) -> String {
    format!("{}timestamp/{}", segment_base_by_shard(shard), time_sec)
}

pub fn index_timestamp_key_prefix(shard: &str) -> String {
    format!("{}timestamp/", segment_base_by_shard(shard))
}

pub fn index_tag_key(shard: &str, tag: String, offset: u64) -> String {
    format!("{}tag/{}/{}", segment_base_by_shard(shard), tag, offset)
}

pub fn index_tag_key_prefix(shard: &str, tag: &str) -> String {
    format!("{}tag/{}/", segment_base_by_shard(shard), tag)
}

pub fn index_key_key(shard: &str, key: String) -> String {
    format!("{}key/{}", segment_base_by_shard(shard), key)
}
