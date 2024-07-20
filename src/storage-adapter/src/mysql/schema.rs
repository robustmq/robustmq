#[derive(Debug, PartialEq, Eq)]
pub struct TMqttRecord {
    pub msgid: String,
    pub header: String,
    pub msg_key: String,
    pub payload: Vec<u8>,
    pub create_time: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TMqttKvMsg {
    pub key: String,
    pub value: Vec<u8>,
    pub create_time: u64,
    pub update_time: u64,
}
