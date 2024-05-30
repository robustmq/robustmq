use axum::async_trait;
use common_base::log::info;
use serde::{Deserialize, Serialize};

use crate::server::tcp::packet::ResponsePackage;

pub mod memory;
pub mod persist;
pub mod ack_manager;

#[derive(Clone,Serialize,Deserialize)]
pub struct QosData {
    pub client_id: String,
    pub create_time: u64,
}

#[derive(Clone)]
pub struct PublishQosMessageData {
    pub client_id: String,
    pub packet: ResponsePackage,
}

#[async_trait]
pub trait QosDataManager {
    // qos
    async fn save_qos_pkid_data(&self, client_id: String, pkid: u16);
    async fn delete_qos_pkid_data(&self, client_id: String, pkid: u16);
    async fn get_qos_pkid_data(&self, client_id: String, pkid: u16) -> Option<QosData>;

    // sub
    async fn save_sub_pkid_data(&self, client_id: String, pkid: u16);
    async fn delete_sub_pkid_data(&self, client_id: String, pkid: u16);
    async fn get_sub_pkid_data(&self, client_id: String, pkid: u16) -> Option<u64>;

    // pub
    async fn save_pub_qos_data(&self, client_id: String, data: PublishQosMessageData);
    async fn delete_pub_qos_data(&self, client_id: String);
    async fn get_pub_qos_data(&self, client_id: String)
        -> Option<PublishQosMessageData>;
}

pub struct QosDataCleanManager {}

impl QosDataCleanManager {
    pub fn new() -> QosDataCleanManager {
        return QosDataCleanManager {};
    }

    pub fn start(&self) {
        info("Idempotent message data cleaning thread started successfully".to_string());
    }
}
