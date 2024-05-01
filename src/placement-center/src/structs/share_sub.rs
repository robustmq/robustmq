use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShareSub {
    pub cluster_name: String,
    pub sub_name: String,
    pub group_name: String,
    pub leader_broker: u64,
    pub create_time: u64,
}
