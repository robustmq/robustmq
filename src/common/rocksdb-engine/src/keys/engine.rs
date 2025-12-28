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
fn segment_base(shard: &str, segment: u32) -> String {
    format!("{}segment/{}/{}/", PREFIX_ENGINE, shard, segment)
}

pub fn offset_segment_cursor_offset(shard: &str, segment: u32) -> String {
    format!("{}cursor/{}/{}", PREFIX_ENGINE, shard, segment)
}

pub fn offset_segment_start(shard: &str, segment: u32) -> String {
    format!("{}offset/start", segment_base(shard, segment))
}

pub fn offset_segment_end(shard: &str, segment: u32) -> String {
    format!("{}offset/end", segment_base(shard, segment))
}

pub fn offset_segment_position(shard: &str, segment: u32, offset: u64) -> String {
    format!("{}offset/position/{}", segment_base(shard, segment), offset)
}

pub fn offset_segment_position_prefix(shard: &str, segment: u32) -> String {
    format!("{}offset/position/", segment_base(shard, segment))
}

pub fn timestamp_segment_start(shard: &str, segment: u32) -> String {
    format!("{}timestamp/start", segment_base(shard, segment))
}

pub fn timestamp_segment_end(shard: &str, segment: u32) -> String {
    format!("{}timestamp/end", segment_base(shard, segment))
}

pub fn timestamp_segment_time(shard: &str, segment: u32, time_sec: u64) -> String {
    format!(
        "{}timestamp/time/{}",
        segment_base(shard, segment),
        time_sec
    )
}

pub fn timestamp_segment_time_prefix(shard: &str, segment: u32) -> String {
    format!("{}timestamp/time/", segment_base(shard, segment))
}

pub fn tag_segment(shard: &str, segment: u32, tag: String, offset: u64) -> String {
    format!("{}tag/{}/{}", segment_base(shard, segment), tag, offset)
}

pub fn tag_segment_prefix(shard: &str, segment: u32, tag: &str) -> String {
    format!("{}tag/{}/", segment_base(shard, segment), tag)
}

pub fn engine_key_index(shard: &str, segment: u32, key: String) -> String {
    format!("{}key/{}", segment_base(shard, segment), key)
}

pub fn segment_index_prefix(shard: &str, segment: u32) -> String {
    segment_base(shard, segment)
}
