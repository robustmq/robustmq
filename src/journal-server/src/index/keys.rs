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

pub(crate) fn offset_segment_start(namespace: &str, shard_name: &str, segment: u32) -> String {
    format!(
        "/index/{}/{}/{}/offset/start",
        namespace, shard_name, segment
    )
}

pub(crate) fn offset_segment_end(namespace: &str, shard_name: &str, segment: u32) -> String {
    format!("/index/{}/{}/{}/offset/end", namespace, shard_name, segment)
}

pub(crate) fn offset_segment_position(
    namespace: &str,
    shard_name: &str,
    segment: u32,
    offset: u64,
) -> String {
    format!(
        "/index/{}/{}/{}/offset/position-{}",
        namespace, shard_name, segment, offset
    )
}

pub(crate) fn timestamp_segment_start(namespace: &str, shard_name: &str, segment: u32) -> String {
    format!(
        "/index/{}/{}/{}/timestamp/start",
        namespace, shard_name, segment
    )
}

pub(crate) fn timestamp_segment_end(namespace: &str, shard_name: &str, segment: u32) -> String {
    format!(
        "/index/{}/{}/{}/timestamp/end",
        namespace, shard_name, segment
    )
}

pub(crate) fn timestamp_segment_time(
    namespace: &str,
    shard_name: &str,
    segment: u32,
    time_sec: u64,
) -> String {
    format!(
        "/index/{}/{}/{}/timestamp/time-{}",
        namespace, shard_name, segment, time_sec
    )
}

pub(crate) fn tag_segment(namespace: &str, shard_name: &str, segment: u32, tag: String) -> String {
    format!(
        "/index/{}/{}/{}/tag/{}",
        namespace, shard_name, segment, tag
    )
}

pub(crate) fn key_segment(namespace: &str, shard_name: &str, segment: u32, key: String) -> String {
    format!(
        "/index/{}/{}/{}/key/{}",
        namespace, shard_name, segment, key
    )
}

pub(crate) fn finish_build_index(namespace: &str, shard_name: &str, segment: u32) -> String {
    format!(
        "/index/{}/{}/{}/build/finish",
        namespace, shard_name, segment
    )
}

pub(crate) fn last_offset_build_index(namespace: &str, shard_name: &str, segment: u32) -> String {
    format!(
        "/index/{}/{}/{}/build/last/offset",
        namespace, shard_name, segment
    )
}

pub(crate) fn segment_index_prefix(namespace: &str, shard_name: &str, segment: u32) -> String {
    format!("/index/{}/{}/{}/", namespace, shard_name, segment)
}
