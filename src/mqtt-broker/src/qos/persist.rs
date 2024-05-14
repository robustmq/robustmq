use axum::async_trait;

use super::{PublishQosMessageData, QosData, QosDataManager};

pub struct QosPersist {}

impl QosPersist {
    pub fn new() -> Self {
        return QosPersist {};
    }
}

#[async_trait]
impl QosDataManager for QosPersist {
    async fn save_qos_pkid_data(&self, client_id: String, pkid: u16) {}

    async fn delete_qos_pkid_data(&self, client_id: String, pkid: u16) {}

    async fn get_qos_pkid_data(&self, client_id: String, pkid: u16) -> Option<QosData> {
        return None;
    }

    async fn save_sub_pkid_data(&self, client_id: String, pkid: u16) {}
    async fn delete_sub_pkid_data(&self, client_id: String, pkid: u16) {}
    async fn get_sub_pkid_data(&self, client_id: String, pkid: u16) -> Option<u64> {
        return None;
    }

    async fn save_pub_qos_data(&self, client_id: String, data: PublishQosMessageData) {}
    async fn delete_pub_qos_data(&self, client_id: String) {}
    async fn get_pub_qos_data(&self, client_id: String) -> Option<PublishQosMessageData> {
        return None;
    }
}
