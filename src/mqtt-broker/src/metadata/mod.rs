use std::default;

use serde::{Deserialize, Serialize};

pub mod cache;
pub mod cluster;
pub mod session;
pub mod user;
pub mod topic;
pub mod subscriber;
pub mod message;
pub mod node;

#[derive(Serialize, Deserialize, Default, Clone)]
pub enum AvailableFlag {
    Enable,
    #[default]
    Disable,
}

impl From<AvailableFlag> for u8 {
    fn from(flag: AvailableFlag) -> Self {
        match flag {
            AvailableFlag::Enable => 1,
            AvailableFlag::Disable => 0,
        }
    }
}
