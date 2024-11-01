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
        "/index/offset/start/{}/{}/{}",
        namespace, shard_name, segment
    )
}

pub(crate) fn offset_segment_end(namespace: &str, shard_name: &str, segment: u32) -> String {
    format!("/index/offset/end/{}/{}/{}", namespace, shard_name, segment)
}

pub(crate) fn offset_index_key(
    namespace: &str,
    shard_name: &str,
    segment: u32,
    offset: u64,
) -> String {
    format!(
        "/index/offset/{}/{}/{}/{}",
        namespace, shard_name, segment, offset
    )
}

pub(crate) fn start_position_index_key(namespace: &str, shard_name: &str, segment: u32) -> String {
    format!(
        "/index/segment/start_position/{}/{}/{}",
        namespace, shard_name, segment
    )
}

pub(crate) fn end_position_index_key(namespace: &str, shard_name: &str, segment: u32) -> String {
    format!(
        "/index/segment/start_position/{}/{}/{}",
        namespace, shard_name, segment
    )
}
