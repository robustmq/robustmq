use protocol::broker_server::generate::mqtt::Available;
use serde::{Deserialize, Serialize};

pub mod acl;
pub mod cache;
pub mod cluster;
pub mod connection;
pub mod message;
pub mod session;
pub mod subscriber;
pub mod topic;
pub mod user;

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

pub fn available_flag(flag: Available) -> AvailableFlag {
    match flag {
        Available::Enable => return AvailableFlag::Enable,
        Available::Disable => return AvailableFlag::Disable,
    }
}
