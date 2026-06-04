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

use std::fmt;

use common_base::error::common::CommonError;
use serde::{Deserialize, Serialize};

/// A struct used for segment status transition in the meta service.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EngineSegment {
    pub shard_name: String,
    pub segment_seq: u32,
    pub replicas: Vec<Replica>,
    pub leader_epoch: u32,
    pub leader: u64,
    pub isr: Vec<u64>,
    pub status: SegmentStatus,

    // ISR protocol
    #[serde(default)]
    pub segment_epoch: u32,
    #[serde(default)]
    pub leader_broker_epoch: u64,
    #[serde(default)]
    pub log_start_offset: u64,
    #[serde(default)]
    pub last_known_isr: Vec<u64>,
}

impl EngineSegment {
    pub fn allow_read(&self) -> bool {
        matches!(
            self.status,
            SegmentStatus::Write
                | SegmentStatus::PreSealUp
                | SegmentStatus::SealUp
                | SegmentStatus::Unavailable
        )
    }

    pub fn allow_write(&self) -> bool {
        matches!(self.status, SegmentStatus::Write | SegmentStatus::PreSealUp)
    }

    pub fn is_replica(&self) -> bool {
        let broker_id = common_config::broker::broker_config().broker_id;
        self.replicas.iter().any(|r| r.node_id == broker_id)
    }

    pub fn get_fold(&self, node_id: u64) -> Option<String> {
        for rep in self.replicas.clone() {
            if rep.node_id == node_id {
                return Some(rep.fold);
            }
        }
        None
    }

    pub fn leader_epoch_incr(&mut self) {
        self.leader_epoch += 1;
    }

    pub fn name(&self) -> String {
        format!("{},{}", self.shard_name, self.segment_seq)
    }

    pub fn encode(&self) -> Result<Vec<u8>, common_base::error::common::CommonError> {
        common_base::utils::serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, common_base::error::common::CommonError> {
        common_base::utils::serialize::deserialize(data)
    }
}

pub fn segment_name(shard_name: &str, segment_no: u32) -> String {
    format!("{shard_name},{segment_no}")
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Replica {
    pub replica_seq: u64,
    pub node_id: u64,
    pub fold: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SegmentStatus {
    #[default]
    Write,
    PreSealUp,
    SealUp,
    PreDelete,
    Deleting,
    // Keep last: bincode encodes variants by index.
    Unavailable,
}

impl fmt::Display for SegmentStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SegmentStatus::Write => write!(f, "Write"),
            SegmentStatus::PreSealUp => write!(f, "PreSealUp"),
            SegmentStatus::SealUp => write!(f, "SealUp"),
            SegmentStatus::PreDelete => write!(f, "PreDelete"),
            SegmentStatus::Deleting => write!(f, "Deleting"),
            SegmentStatus::Unavailable => write!(f, "Unavailable"),
        }
    }
}

pub fn str_to_segment_status(status: &str) -> Result<SegmentStatus, CommonError> {
    match status {
        "Write" => Ok(SegmentStatus::Write),
        "PreSealUp" => Ok(SegmentStatus::PreSealUp),
        "SealUp" => Ok(SegmentStatus::SealUp),
        "PreDelete" => Ok(SegmentStatus::PreDelete),
        "Deleting" => Ok(SegmentStatus::Deleting),
        "Unavailable" => Ok(SegmentStatus::Unavailable),
        _ => Err(CommonError::CommonError(format!(
            "Invalid segment status '{}'. Valid values are: Write, PreSealUp, SealUp, PreDelete, Deleting, Unavailable",
            status
        ))),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SegmentConfig {
    pub max_segment_size: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allow_read_write_matrix() {
        let mut seg = EngineSegment::default();

        for (status, read, write) in [
            (SegmentStatus::Write, true, true),
            (SegmentStatus::PreSealUp, true, true),
            (SegmentStatus::SealUp, true, false),
            (SegmentStatus::Unavailable, true, false),
            (SegmentStatus::PreDelete, false, false),
            (SegmentStatus::Deleting, false, false),
        ] {
            seg.status = status.clone();
            assert_eq!(seg.allow_read(), read, "allow_read for {status}");
            assert_eq!(seg.allow_write(), write, "allow_write for {status}");
        }
    }

    #[test]
    fn segment_status_string_roundtrip() {
        for status in [
            SegmentStatus::Write,
            SegmentStatus::PreSealUp,
            SegmentStatus::SealUp,
            SegmentStatus::PreDelete,
            SegmentStatus::Deleting,
            SegmentStatus::Unavailable,
        ] {
            let s = status.to_string();
            assert_eq!(str_to_segment_status(&s).unwrap(), status);
        }
        assert!(str_to_segment_status("Nope").is_err());
    }

    #[test]
    fn new_isr_fields_default_to_zero() {
        let seg = EngineSegment::default();
        assert_eq!(seg.segment_epoch, 0);
        assert_eq!(seg.leader_broker_epoch, 0);
        assert_eq!(seg.log_start_offset, 0);
        assert!(seg.last_known_isr.is_empty());
    }

    #[test]
    fn encode_decode_roundtrip_with_isr_fields() {
        let seg = EngineSegment {
            shard_name: "s1".to_string(),
            segment_seq: 2,
            leader: 7,
            leader_epoch: 3,
            isr: vec![7, 8, 9],
            status: SegmentStatus::Unavailable,
            segment_epoch: 11,
            leader_broker_epoch: 42,
            log_start_offset: 100,
            last_known_isr: vec![7, 8],
            ..Default::default()
        };
        let decoded = EngineSegment::decode(&seg.encode().unwrap()).unwrap();
        assert_eq!(decoded.segment_epoch, 11);
        assert_eq!(decoded.leader_broker_epoch, 42);
        assert_eq!(decoded.log_start_offset, 100);
        assert_eq!(decoded.last_known_isr, vec![7, 8]);
        assert_eq!(decoded.status, SegmentStatus::Unavailable);
    }
}
