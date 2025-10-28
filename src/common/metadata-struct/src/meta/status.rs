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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct MetaStatus {
    pub running_state: RunningState,
    pub id: u64,
    pub current_term: u64,
    pub vote: Vote,
    pub last_log_index: u64,
    pub last_applied: LastApplied,
    pub snapshot: Option<serde_json::Value>,
    pub purged: Option<serde_json::Value>,
    pub state: String,
    pub current_leader: u64,
    pub millis_since_quorum_ack: u64,
    pub last_quorum_acked: u128,
    pub membership_config: MembershipConfig,
    pub heartbeat: HashMap<String, u128>,
    pub replication: HashMap<String, ReplicationState>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct RunningState {
    #[serde(rename = "Ok")]
    pub ok: Option<serde_json::Value>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Vote {
    pub leader_id: LeaderId,
    pub committed: bool,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct LeaderId {
    pub term: u64,
    pub node_id: u64,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct LastApplied {
    pub leader_id: LeaderId,
    pub index: u64,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct MembershipConfig {
    pub log_id: LogId,
    pub membership: Membership,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct LogId {
    pub leader_id: LeaderId,
    pub index: u64,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Membership {
    pub configs: Vec<Vec<u64>>,
    pub nodes: HashMap<String, NodeInfo>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: u64,
    pub rpc_addr: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ReplicationState {
    pub leader_id: LeaderId,
    pub index: u64,
}

impl MetaStatus {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }

    pub fn is_leader(&self) -> bool {
        self.state == "Leader"
    }

    pub fn is_follower(&self) -> bool {
        self.state == "Follower"
    }

    pub fn is_candidate(&self) -> bool {
        self.state == "Candidate"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_status_deserialization() {
        let json_data = r#"{"running_state":{"Ok":null},"id":1,"current_term":1,"vote":{"leader_id":{"term":1,"node_id":1},"committed":true},"last_log_index":422,"last_applied":{"leader_id":{"term":1,"node_id":1},"index":422},"snapshot":null,"purged":null,"state":"Leader","current_leader":1,"millis_since_quorum_ack":0,"last_quorum_acked":1760828146763525625,"membership_config":{"log_id":{"leader_id":{"term":0,"node_id":0},"index":0},"membership":{"configs":[[1]],"nodes":{"1":{"node_id":1,"rpc_addr":"127.0.0.1:1228"}}}},"heartbeat":{"1":1760828146387602084},"replication":{"1":{"leader_id":{"term":1,"node_id":1},"index":422}}}"#;

        let status: MetaStatus = serde_json::from_str(json_data).unwrap();

        assert_eq!(status.id, 1);
        assert_eq!(status.current_term, 1);
        assert_eq!(status.state, "Leader");
        assert_eq!(status.current_leader, 1);
        assert_eq!(status.last_log_index, 422);
        assert!(status.is_leader());
        assert!(!status.is_follower());
        assert!(!status.is_candidate());
        assert_eq!(status.vote.leader_id.node_id, 1);
        assert!(status.vote.committed);
    }

    #[test]
    fn test_meta_status_serialization() {
        let json_data = r#"{"running_state":{"Ok":null},"id":1,"current_term":1,"vote":{"leader_id":{"term":1,"node_id":1},"committed":true},"last_log_index":422,"last_applied":{"leader_id":{"term":1,"node_id":1},"index":422},"snapshot":null,"purged":null,"state":"Leader","current_leader":1,"millis_since_quorum_ack":0,"last_quorum_acked":1760828146763525625,"membership_config":{"log_id":{"leader_id":{"term":0,"node_id":0},"index":0},"membership":{"configs":[[1]],"nodes":{"1":{"node_id":1,"rpc_addr":"127.0.0.1:1228"}}}},"heartbeat":{"1":1760828146387602084},"replication":{"1":{"leader_id":{"term":1,"node_id":1},"index":422}}}"#;

        let status: MetaStatus = serde_json::from_str(json_data).unwrap();
        let encoded = status.encode();
        let decoded: MetaStatus = serde_json::from_slice(&encoded).unwrap();

        assert_eq!(decoded.id, status.id);
        assert_eq!(decoded.current_term, status.current_term);
        assert_eq!(decoded.state, status.state);
    }
}
