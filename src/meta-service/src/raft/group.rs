use openraft::Raft;

use crate::{core::error::MetaServiceError, raft::type_config::TypeConfig};

pub struct MultiRaftManager {
    group_num: u32,
    pub raft_group: Vec<Raft<TypeConfig>>,
}

impl MultiRaftManager {
    pub fn new(group_num: u32) -> Self {
        MultiRaftManager {
            group_num,
            raft_group: Vec::new(),
        }
    }

    pub fn start() -> Result<(), MetaServiceError> {
        Ok(())
    }

}
