use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct MQTTBlackList {
    pub black_list_type: MQTTBlackListType,
    pub resource_name: String,
    pub end_time: u64,
    pub dec: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub enum MQTTBlackListType {
    ClientId,
    User,
    IP,
}
