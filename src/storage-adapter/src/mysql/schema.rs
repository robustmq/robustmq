#[derive(Debug, PartialEq, Eq)]
pub struct TMqttStreamMsg {
    pub id: u32,
    pub msgid: String,
    pub shard: String,
    pub qos: u16,
    pub payload: String,
    pub create_time: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TMqttKvMsg {
    pub key: String,
    pub value: String,
    pub create_time: u64,
}
