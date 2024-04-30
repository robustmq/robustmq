#[derive(Clone)]
pub struct ShareSub {
    pub cluster_name: String,
    pub share_sub_name: String,
    pub topics: String,
    pub leader_broker: u64,
    pub create_time: u64,
}
