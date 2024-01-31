pub mod broker;

pub enum BrokerActionType{
    RegisterBroker,
    UnRegisterBroker,
}

pub struct BrokerNode{
    pub broker_id: u64,
    pub broker_ip: String
}