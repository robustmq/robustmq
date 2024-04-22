#[derive(Debug, PartialEq, Eq)]
pub struct TMqttRecord {
    pub msgid: String,
    pub payload: Vec<u8>,
    pub create_time: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TMqttKvMsg {
    pub key: String,
    pub value: Vec<u8>,
    pub create_time: u64,
}
