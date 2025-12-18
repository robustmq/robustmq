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

use crate::segment::storage::SegmentIdentity;

pub(crate) fn offset_segment_start(segment_iden: &SegmentIdentity) -> String {
    format!(
        "/index/{}/{}/offset/start",
        segment_iden.shard_name, segment_iden.segment_seq,
    )
}

pub(crate) fn offset_segment_end(segment_iden: &SegmentIdentity) -> String {
    format!(
        "/index/{}/{}/offset/end",
        segment_iden.shard_name, segment_iden.segment_seq,
    )
}

pub(crate) fn offset_segment_position(segment_iden: &SegmentIdentity, offset: u64) -> String {
    format!(
        "/index/{}/{}/offset/position-{}",
        segment_iden.shard_name, segment_iden.segment_seq, offset
    )
}

pub(crate) fn offset_segment_position_prefix(segment_iden: &SegmentIdentity) -> String {
    format!(
        "/index/{}/{}/offset/position-",
        segment_iden.shard_name, segment_iden.segment_seq
    )
}

pub(crate) fn timestamp_segment_start(segment_iden: &SegmentIdentity) -> String {
    format!(
        "/index/{}/{}/timestamp/start",
        segment_iden.shard_name, segment_iden.segment_seq
    )
}

pub(crate) fn timestamp_segment_end(segment_iden: &SegmentIdentity) -> String {
    format!(
        "/index/{}/{}/timestamp/end",
        segment_iden.shard_name, segment_iden.segment_seq
    )
}

pub(crate) fn timestamp_segment_time(segment_iden: &SegmentIdentity, time_sec: u64) -> String {
    format!(
        "/index/{}/{}/timestamp/time-{}",
        segment_iden.shard_name, segment_iden.segment_seq, time_sec
    )
}

pub(crate) fn timestamp_segment_time_prefix(segment_iden: &SegmentIdentity) -> String {
    format!(
        "/index/{}/{}/timestamp/time-",
        segment_iden.shard_name, segment_iden.segment_seq
    )
}

pub(crate) fn tag_segment(segment_iden: &SegmentIdentity, tag: String, offset: u64) -> String {
    format!(
        "/index/{}/{}/tag/{}/{}",
        segment_iden.shard_name, segment_iden.segment_seq, tag, offset
    )
}
pub(crate) fn tag_segment_prefix(segment_iden: &SegmentIdentity, tag: String) -> String {
    format!(
        "/index/{}/{}/tag/{}/",
        segment_iden.shard_name, segment_iden.segment_seq, tag
    )
}

pub(crate) fn key_segment(segment_iden: &SegmentIdentity, key: String, offset: u64) -> String {
    format!(
        "/index/{}/{}/key/{}/{}",
        segment_iden.shard_name, segment_iden.segment_seq, key, offset
    )
}

pub(crate) fn key_segment_prefix(segment_iden: &SegmentIdentity, key: String) -> String {
    format!(
        "/index/{}/{}/key/{}/",
        segment_iden.shard_name, segment_iden.segment_seq, key
    )
}

pub(crate) fn finish_build_index(segment_iden: &SegmentIdentity) -> String {
    format!(
        "/index/{}/{}/build/finish",
        segment_iden.shard_name, segment_iden.segment_seq,
    )
}

pub(crate) fn last_offset_build_index(segment_iden: &SegmentIdentity) -> String {
    format!(
        "/index/{}/{}/build/last/offset",
        segment_iden.shard_name, segment_iden.segment_seq,
    )
}

pub(crate) fn segment_index_prefix(segment_iden: &SegmentIdentity) -> String {
    format!(
        "/index/{}/{}/",
        segment_iden.shard_name, segment_iden.segment_seq,
    )
}
