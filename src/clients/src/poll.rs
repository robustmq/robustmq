use crate::placement::{
    journal::JournalServiceManager, kv::KvServiceManager, mqtt::MQTTServiceManager,
    placement::PlacementServiceManager,
};
use common_base::errors::RobustMQError;
use dashmap::DashMap;
use mobc::Pool;
use protocol::placement_center::generate::{
    journal::engine_service_client::EngineServiceClient, kv::kv_service_client::KvServiceClient,
    mqtt::mqtt_service_client::MqttServiceClient,
    placement::placement_center_service_client::PlacementCenterServiceClient,
};
use tonic::transport::Channel;

#[derive(Clone)]
pub struct ClientPool {
    ip_max_num: u64,
    placement_service_pools: DashMap<String, Pool<PlacementServiceManager>>,
    journal_service_pools: DashMap<String, Pool<JournalServiceManager>>,
    kv_service_pools: DashMap<String, Pool<KvServiceManager>>,
    mqtt_service_pools: DashMap<String, Pool<MQTTServiceManager>>,
}

impl ClientPool {
    pub fn new(ip_max_num: u64) -> Self {
        Self {
            ip_max_num,
            placement_service_pools: DashMap::with_capacity(128),
            journal_service_pools: DashMap::with_capacity(128),
            kv_service_pools: DashMap::with_capacity(128),
            mqtt_service_pools: DashMap::with_capacity(128),
        }
    }

    pub async fn get_placement_services_client(
        &self,
        addr: String,
    ) -> Result<PlacementCenterServiceClient<Channel>, RobustMQError> {
        if !self.placement_service_pools.contains_key(&addr) {
            let manager = PlacementServiceManager::new(addr.clone());
            let pool = Pool::builder().max_open(self.ip_max_num).build(manager);
            self.placement_service_pools
                .insert(addr.clone(), pool);
        }
        if let Some(poll) = self.placement_service_pools.get(&addr) {
            match poll.get().await {
                Ok(conn) => return Ok(conn.into_inner()),
                Err(e) => {
                    return Err(RobustMQError::NoAvailableConnection(e.to_string()));
                }
            };
        }
        return Err(RobustMQError::NoAvailableConnection(
            "Placement service client connection pool in the placement center has no connections available".to_string(),));
    }

    pub async fn get_journal_services_client(
        &self,
        addr: String,
    ) -> Result<EngineServiceClient<Channel>, RobustMQError> {
        if !self.journal_service_pools.contains_key(&addr) {
            let manager = JournalServiceManager::new(addr.clone());
            let pool = Pool::builder().max_open(self.ip_max_num).build(manager);
            self.journal_service_pools.insert(addr.clone(), pool);
        }
        if let Some(poll) = self.journal_service_pools.get(&addr) {
            match poll.get().await {
                Ok(conn) => {
                    return Ok(conn.into_inner());
                }
                Err(e) => {
                    return Err(RobustMQError::NoAvailableConnection(e.to_string()));
                }
            };
        }
        return Err(RobustMQError::NoAvailableConnection(
            "Journal service client connection pool in the placement center has no connections available".to_string(),));
    }

    pub async fn get_kv_services_client(
        &self,
        addr: String,
    ) -> Result<KvServiceClient<Channel>, RobustMQError> {
        if !self.kv_service_pools.contains_key(&addr) {
            let manager = KvServiceManager::new(addr.clone());
            let pool = Pool::builder().max_open(self.ip_max_num).build(manager);
            self.kv_service_pools.insert(addr.clone(), pool);
        }
        if let Some(poll) = self.kv_service_pools.get(&addr) {
            match poll.get().await {
                Ok(conn) => {
                    return Ok(conn.into_inner());
                }
                Err(e) => {
                    return Err(RobustMQError::NoAvailableConnection(e.to_string()));
                }
            };
        }
        return Err(RobustMQError::NoAvailableConnection(
            "Kv service client connection pool in the placement center has no connections available".to_string(),
        ));
    }

    pub async fn get_mqtt_services_client(
        &self,
        addr: String,
    ) -> Result<MqttServiceClient<Channel>, RobustMQError> {
        if !self.kv_service_pools.contains_key(&addr) {
            let manager = MQTTServiceManager::new(addr.clone());
            let pool = Pool::builder().max_open(self.ip_max_num).build(manager);
            self.mqtt_service_pools.insert(addr.clone(), pool);
        }
        if let Some(poll) = self.mqtt_service_pools.get(&addr) {
            match poll.get().await {
                Ok(conn) => {
                    return Ok(conn.into_inner());
                }
                Err(e) => {
                    return Err(RobustMQError::NoAvailableConnection(e.to_string()));
                }
            };
        }
        return Err(RobustMQError::NoAvailableConnection(
            "Mqtt service client connection pool in the placement center has no connections available".to_string(),
        ));
    }
}
