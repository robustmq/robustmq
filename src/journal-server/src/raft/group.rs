use dashmap::DashMap;

use super::machine::SegmentStatusMachine;

pub struct RaftGroupManager {
    pub segment_groups: DashMap<String, SegmentStatusMachine>,
}

impl RaftGroupManager {
    pub fn new() -> Self {
        let segment_groups = DashMap::new();
        return RaftGroupManager { segment_groups };
    }
}
