use crate::{
    mqtt::MqttBrokerMqttServiceManager,
    placement::{
        journal::JournalServiceManager, kv::KvServiceManager, mqtt::MQTTServiceManager,
        placement::PlacementServiceManager,
    },
};
use common_base::errors::RobustMQError;
use dashmap::DashMap;
use mobc::Pool;
use protocol::{
    broker_server::generate::mqtt::mqtt_broker_service_client::MqttBrokerServiceClient,
    placement_center::generate::{
        journal::engine_service_client::EngineServiceClient,
        kv::kv_service_client::KvServiceClient, mqtt::mqtt_service_client::MqttServiceClient,
        placement::placement_center_service_client::PlacementCenterServiceClient,
    },
};
use tonic::transport::Channel;

#[derive(Clone)]
pub struct ClientPool {
    max_open_connection: u64,
    // placement center
    placement_center_inner_pools: DashMap<String, Pool<PlacementServiceManager>>,
    placement_center_journal_service_pools: DashMap<String, Pool<JournalServiceManager>>,
    placement_center_kv_service_pools: DashMap<String, Pool<KvServiceManager>>,
    placement_center_mqtt_service_pools: DashMap<String, Pool<MQTTServiceManager>>,

    // mqtt broker
    mqtt_broker_mqtt_service_pools: DashMap<String, Pool<MqttBrokerMqttServiceManager>>,
}

impl ClientPool {
    pub fn new(max_open_connection: u64) -> Self {
        Self {
            max_open_connection,
            placement_center_inner_pools: DashMap::with_capacity(128),
            placement_center_journal_service_pools: DashMap::with_capacity(128),
            placement_center_kv_service_pools: DashMap::with_capacity(128),
            placement_center_mqtt_service_pools: DashMap::with_capacity(128),
            mqtt_broker_mqtt_service_pools: DashMap::with_capacity(128),
        }
    }

    pub async fn get_placement_center_inner_services_client(
        &self,
        addr: String,
    ) -> Result<PlacementCenterServiceClient<Channel>, RobustMQError> {
        let module = "PlacementService".to_string();
        let key = format!("{}_{}_{}", "PlacementServer", module, addr);
        if !self.placement_center_inner_pools.contains_key(&key) {
            let manager = PlacementServiceManager::new(addr.clone());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.placement_center_inner_pools.insert(key.clone(), pool);
        }
        if let Some(poll) = self.placement_center_inner_pools.get(&key) {
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

    pub async fn placement_center_journal_services_client(
        &self,
        addr: String,
    ) -> Result<EngineServiceClient<Channel>, RobustMQError> {
        let module = "JournalService".to_string();
        let key = format!("{}_{}_{}", "JournalServer", module, addr);
        if !self
            .placement_center_journal_service_pools
            .contains_key(&key)
        {
            let manager = JournalServiceManager::new(addr.clone());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.placement_center_journal_service_pools
                .insert(key.clone(), pool);
        }
        if let Some(poll) = self.placement_center_journal_service_pools.get(&key) {
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

    pub async fn placement_center_kv_services_client(
        &self,
        addr: String,
    ) -> Result<KvServiceClient<Channel>, RobustMQError> {
        let module = "KvServices".to_string();
        let key = format!("{}_{}_{}", "PlacementCenter", module, addr);

        if !self.placement_center_kv_service_pools.contains_key(&key) {
            let manager = KvServiceManager::new(addr.clone());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.placement_center_kv_service_pools
                .insert(key.clone(), pool);
        }

        if let Some(poll) = self.placement_center_kv_service_pools.get(&key) {
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

    pub async fn placement_center_mqtt_services_client(
        &self,
        addr: String,
    ) -> Result<MqttServiceClient<Channel>, RobustMQError> {
        let module = "PlacementMqttServices".to_string();
        let key = format!("{}_{}_{}", "PlacementCenter", module, addr);

        if !self.placement_center_kv_service_pools.contains_key(&key) {
            let manager = MQTTServiceManager::new(addr.clone());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.placement_center_mqtt_service_pools
                .insert(key.clone(), pool);
        }

        if let Some(poll) = self.placement_center_mqtt_service_pools.get(&key) {
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

    pub async fn mqtt_broker_mqtt_services_client(
        &self,
        addr: String,
    ) -> Result<MqttBrokerServiceClient<Channel>, RobustMQError> {
        let module = "MqttServices".to_string();
        let key = format!("{}_{}_{}", "MQTTBroker", module, addr);

        if !self.mqtt_broker_mqtt_service_pools.contains_key(&key) {
            let manager = MqttBrokerMqttServiceManager::new(addr.clone());
            let pool = Pool::builder()
                .max_open(self.max_open_connection)
                .build(manager);
            self.mqtt_broker_mqtt_service_pools
                .insert(key.clone(), pool);
        }

        if let Some(poll) = self.mqtt_broker_mqtt_service_pools.get(&key) {
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
