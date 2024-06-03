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
        let module = "PlacementService".to_string();
        let key = format!("{}_{}_{}", "PlacementServer", module, addr);
        if !self.placement_service_pools.contains_key(&key) {
            let manager = PlacementServiceManager::new(addr.clone());
            let pool = Pool::builder().max_open(self.ip_max_num).build(manager);
            self.placement_service_pools.insert(key.clone(), pool);
        }
        if let Some(poll) = self.placement_service_pools.get(&key) {
            match poll.get().await {
                Ok(conn) => return Ok(conn.into_inner()),
                Err(e) => {
                    return Err(RobustMQError::NoAvailableConnection(module, e.to_string()));
                }
            };
        }
        return Err(RobustMQError::NoAvailableConnection(
            module,
            "connection pool is not initialized".to_string(),
        ));
    }

    pub async fn get_journal_services_client(
        &self,
        addr: String,
    ) -> Result<EngineServiceClient<Channel>, RobustMQError> {
        let module = "JournalService".to_string();
        let key = format!("{}_{}_{}", "JournalServer", module, addr);
        if !self.journal_service_pools.contains_key(&key) {
            let manager = JournalServiceManager::new(addr.clone());
            let pool = Pool::builder().max_open(self.ip_max_num).build(manager);
            self.journal_service_pools.insert(key.clone(), pool);
        }
        if let Some(poll) = self.journal_service_pools.get(&key) {
            match poll.get().await {
                Ok(conn) => {
                    return Ok(conn.into_inner());
                }
                Err(e) => {
                    return Err(RobustMQError::NoAvailableConnection(module, e.to_string()));
                }
            };
        }
        return Err(RobustMQError::NoAvailableConnection(
            module,
            "connection pool is not initialized".to_string(),
        ));
    }

    pub async fn get_kv_services_client(
        &self,
        addr: String,
    ) -> Result<KvServiceClient<Channel>, RobustMQError> {
        let module = "KvServices".to_string();
        let key = format!("{}_{}_{}", "PlacementCenter", module, addr);

        if !self.kv_service_pools.contains_key(&key) {
            let manager = KvServiceManager::new(addr.clone());
            let pool = Pool::builder().max_open(self.ip_max_num).build(manager);
            self.kv_service_pools.insert(key.clone(), pool);
        }

        if let Some(poll) = self.kv_service_pools.get(&key) {
            match poll.get().await {
                Ok(conn) => {
                    return Ok(conn.into_inner());
                }
                Err(e) => {
                    return Err(RobustMQError::NoAvailableConnection(module, e.to_string()));
                }
            };
        }

        return Err(RobustMQError::NoAvailableConnection(
            module,
            "connection pool is not initialized".to_string(),
        ));
    }

    pub async fn get_mqtt_services_client(
        &self,
        addr: String,
    ) -> Result<MqttServiceClient<Channel>, RobustMQError> {
        let module = "MqttServices".to_string();
        let key = format!("{}_{}_{}", "PlacementCenter", module, addr);

        if !self.kv_service_pools.contains_key(&key) {
            let manager = MQTTServiceManager::new(addr.clone());
            let pool = Pool::builder().max_open(self.ip_max_num).build(manager);
            self.mqtt_service_pools.insert(key.clone(), pool);
        }

        if let Some(poll) = self.mqtt_service_pools.get(&key) {
            match poll.get().await {
                Ok(conn) => {
                    return Ok(conn.into_inner());
                }
                Err(e) => {
                    return Err(RobustMQError::NoAvailableConnection(module, e.to_string()));
                }
            };
        }
        return Err(RobustMQError::NoAvailableConnection(
            module,
            "connection pool is not initialized".to_string(),
        ));
    }
}
