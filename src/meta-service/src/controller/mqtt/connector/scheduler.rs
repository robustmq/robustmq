// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::status::ConnectorContext;
use crate::{
    controller::mqtt::call_broker::MQTTInnerCallManager,
    core::{cache::CacheManager, error::MetaServiceError},
    raft::route::apply::StorageDriver,
};
use common_base::{
    error::ResultCommonError,
    tools::{loop_select_ticket, now_second},
};
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::bridge::{connector::MQTTConnector, status::MQTTStatus};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;
use tracing::{info, warn};

/// Connector Scheduler - Manages MQTT connector lifecycle and load balancing
pub struct ConnectorScheduler {
    cache_manager: Arc<CacheManager>,
    heartbeat_timeout_sec: u64,
    connector_context: ConnectorContext,
}

impl ConnectorScheduler {
    pub fn new(
        raft_machine_apply: Arc<StorageDriver>,
        call_manager: Arc<MQTTInnerCallManager>,
        client_pool: Arc<ClientPool>,
        cache_manager: Arc<CacheManager>,
    ) -> Self {
        let config = broker_config();
        let connector_context = ConnectorContext::new(
            raft_machine_apply,
            call_manager,
            client_pool,
            cache_manager.clone(),
        );
        Self {
            cache_manager,
            heartbeat_timeout_sec: config.meta_runtime.heartbeat_timeout_ms / 1000,
            connector_context,
        }
    }

    pub async fn run(&self, stop_send: broadcast::Sender<bool>) {
        let scheduler = self;
        let ac_fn = async move || -> ResultCommonError {
            scheduler.scheduler_cycle().await;
            Ok(())
        };

        loop_select_ticket(ac_fn, 1, &stop_send).await;
    }
}

pub async fn start_connector_scheduler(
    cache_manager: &Arc<CacheManager>,
    raft_machine_apply: &Arc<StorageDriver>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    stop_send: broadcast::Sender<bool>,
) {
    let scheduler = ConnectorScheduler::new(
        raft_machine_apply.clone(),
        call_manager.clone(),
        client_pool.clone(),
        cache_manager.clone(),
    );

    scheduler.run(stop_send).await;
}

impl ConnectorScheduler {
    async fn scheduler_cycle(&self) {
        let start_time = now_second();

        // Execute heartbeat check and connector management concurrently
        let (heartbeat_result, connector_result) =
            tokio::join!(self.check_heartbeat(), self.start_stop_connector_thread());

        // Handle results
        let mut has_error = false;

        if let Err(e) = heartbeat_result {
            warn!(
                "Heartbeat check failed: {:?}. This may cause connectors to not be rescheduled on timeout.",
                e
            );
            has_error = true;
        }

        if let Err(e) = connector_result {
            warn!(
                "Connector scheduling failed: {:?}. New connectors may not be started.",
                e
            );
            has_error = true;
        }

        let elapsed = now_second() - start_time;
        if elapsed > 5 {
            warn!(
                "Scheduler cycle took {}s (>5s threshold). System may be overloaded.",
                elapsed
            );
        } else if !has_error {
            // Only log success when no errors occurred and execution was fast
            if elapsed > 1 {
                info!("Scheduler cycle completed in {}s", elapsed);
            }
        }
    }

    async fn check_heartbeat(&self) -> Result<(), MetaServiceError> {
        let current_time = now_second();

        let mut orphaned_count = 0;
        let mut expired_count = 0;

        for heartbeat in self.cache_manager.get_all_connector_heartbeat() {
            // Check if connector still exists
            let connector = match self
                .cache_manager
                .get_connector(&heartbeat.cluster_name, &heartbeat.connector_name)
            {
                Some(c) => c,
                None => {
                    // Connector was deleted, clean up orphaned heartbeat
                    warn!(
                        "Removing orphaned heartbeat for non-existent connector: cluster={}, name={}",
                        heartbeat.cluster_name, heartbeat.connector_name
                    );
                    self.cache_manager.remove_connector_heartbeat(
                        &heartbeat.cluster_name,
                        &heartbeat.connector_name,
                    );
                    orphaned_count += 1;
                    continue;
                }
            };

            // Check if heartbeat expired
            let elapsed_time = current_time - heartbeat.last_heartbeat;
            if elapsed_time > self.heartbeat_timeout_sec {
                warn!(
                    "Connector heartbeat expired: cluster={}, name={}, broker_id={:?}, elapsed={}s, timeout={}s",
                    connector.cluster_name,
                    connector.connector_name,
                    connector.broker_id,
                    elapsed_time,
                    self.heartbeat_timeout_sec
                );

                // Reset connector to Idle for rescheduling
                if let Err(e) = self
                    .connector_context
                    .update_status_to_idle(&heartbeat.cluster_name, &heartbeat.connector_name)
                    .await
                {
                    warn!(
                        "Failed to reset expired connector {} to Idle: {:?}",
                        connector.connector_name, e
                    );
                } else {
                    expired_count += 1;
                }
            }
        }

        if orphaned_count > 0 || expired_count > 0 {
            info!(
                "Heartbeat check completed: {} orphaned heartbeats removed, {} expired connectors reset",
                orphaned_count, expired_count
            );
        }

        Ok(())
    }

    async fn start_stop_connector_thread(&self) -> Result<(), MetaServiceError> {
        // Step 1: Group connectors by cluster for batch broker assignment
        let mut unassigned_connectors_by_cluster: HashMap<String, Vec<_>> = HashMap::new();
        let mut connectors_to_process = Vec::new();

        for connector in self.cache_manager.get_all_connector() {
            // Detect and fix abnormal state: Running without broker_id
            if connector.broker_id.is_none() && connector.status == MQTTStatus::Running {
                warn!(
                    "Connector {} has abnormal state (Running without broker_id), resetting to Idle",
                    connector.connector_name
                );
                if let Err(e) = self
                    .connector_context
                    .update_status_to_idle(&connector.cluster_name, &connector.connector_name)
                    .await
                {
                    warn!(
                        "Failed to reset abnormal connector {} to Idle: {:?}",
                        connector.connector_name, e
                    );
                }
                continue;
            }

            if connector.broker_id.is_none() {
                unassigned_connectors_by_cluster
                    .entry(connector.cluster_name.clone())
                    .or_default()
                    .push(connector);
            } else {
                connectors_to_process.push(connector);
            }
        }

        // Step 2: Batch assign brokers for unassigned connectors
        for (cluster_name, connectors) in unassigned_connectors_by_cluster {
            if let Err(e) = self.assign_brokers_batch(&cluster_name, connectors).await {
                warn!(
                    "Failed to assign brokers for cluster {}: {:?}",
                    cluster_name, e
                );
            }
        }

        // Step 3: Process connectors with assigned brokers
        for connector in connectors_to_process {
            let connector_name = connector.connector_name.clone();
            if let Err(e) = self.process_connector_state(connector).await {
                warn!("Failed to process connector {}: {:?}", connector_name, e);
            }
        }

        Ok(())
    }

    async fn assign_brokers_batch(
        &self,
        cluster_name: &str,
        connectors: Vec<MQTTConnector>,
    ) -> Result<(), MetaServiceError> {
        if connectors.is_empty() {
            return Ok(());
        }

        // Step 1: Calculate initial broker load (once for all connectors)
        let mut broker_load = calculate_broker_load_internal(&self.cache_manager, cluster_name)?;

        // Step 2: Assign each connector to the broker with minimum load
        for mut connector in connectors {
            // Select broker with minimum load
            let broker_id = match broker_load
                .iter()
                .min_by_key(|(_, count)| *count)
                .map(|(id, _)| *id)
            {
                Some(id) => id,
                None => {
                    warn!(
                        "No available broker for connector {}: cluster has no brokers",
                        connector.connector_name
                    );
                    continue;
                }
            };

            connector.broker_id = Some(broker_id);
            connector.status = MQTTStatus::Idle;

            info!(
                "Connector {} assigned to Broker {} for execution (current load: {})",
                connector.connector_name,
                broker_id,
                broker_load.get(&broker_id).unwrap_or(&0)
            );

            // Update local load counter immediately (optimization: avoid recalculating)
            *broker_load.entry(broker_id).or_insert(0) += 1;

            if let Err(e) = self
                .connector_context
                .save_connector(connector.clone())
                .await
            {
                warn!(
                    "Failed to save connector {} assignment: {:?}",
                    connector.connector_name, e
                );
                // Rollback local load counter on save failure
                if let Some(count) = broker_load.get_mut(&broker_id) {
                    *count = count.saturating_sub(1);
                }
            }
        }

        Ok(())
    }

    async fn process_connector_state(
        &self,
        connector: MQTTConnector,
    ) -> Result<(), MetaServiceError> {
        match connector.status {
            MQTTStatus::Running => {
                // Already running, no action needed
                Ok(())
            }
            MQTTStatus::Idle => {
                self.connector_context
                    .update_status_to_running(&connector.cluster_name, &connector.connector_name)
                    .await
            }
        }
    }
}

/// Calculate broker load - extracted for testing
fn calculate_broker_load_internal(
    cache_manager: &CacheManager,
    cluster_name: &str,
) -> Result<HashMap<u64, usize>, MetaServiceError> {
    // Step 1: Initialize all brokers with 0 load
    let mut broker_load: HashMap<u64, usize> = cache_manager
        .get_broker_node_id_by_cluster(cluster_name)
        .into_iter()
        .map(|broker_id| (broker_id, 0))
        .collect();

    if broker_load.is_empty() {
        return Err(MetaServiceError::NoAvailableBrokerNode);
    }

    // Step 2: Count connectors for each broker
    for connector in cache_manager.get_all_connector() {
        if let Some(broker_id) = connector.broker_id {
            *broker_load.entry(broker_id).or_insert(0) += 1;
        }
    }

    Ok(broker_load)
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_base::tools::now_second;
    use metadata_struct::meta::node::BrokerNode;
    use metadata_struct::mqtt::bridge::connector::{FailureHandlingStrategy, MQTTConnector};
    use metadata_struct::mqtt::bridge::connector_type::ConnectorType;
    use metadata_struct::mqtt::bridge::status::MQTTStatus;
    use rocksdb_engine::test::test_rocksdb_instance;

    /// Setup test cluster with specified brokers and connector distribution
    /// Returns (cache_manager, cluster_name)
    fn setup_test_cluster(
        broker_count: usize,
        connector_distribution: Vec<usize>, // connectors per broker
    ) -> (Arc<CacheManager>, String) {
        let cache_manager = Arc::new(CacheManager::new(test_rocksdb_instance()));
        let cluster_name = "test_cluster".to_string();

        // Add brokers
        for i in 1..=broker_count {
            cache_manager.add_broker_node(BrokerNode {
                cluster_name: cluster_name.clone(),
                node_id: i as u64,
                node_ip: format!("127.0.0.{}", i),
                node_inner_addr: format!("127.0.0.{}:9000", i),
                extend: "{}".to_string(),
                roles: Vec::new(),
                start_time: now_second(),
                register_time: now_second(),
            });
        }

        // Add connectors
        for (broker_idx, &connector_count) in connector_distribution.iter().enumerate() {
            let broker_id = (broker_idx + 1) as u64;
            for j in 0..connector_count {
                cache_manager.add_connector(
                    &cluster_name,
                    &MQTTConnector {
                        cluster_name: cluster_name.clone(),
                        connector_name: format!("conn_b{}_n{}", broker_id, j),
                        connector_type: ConnectorType::LocalFile,
                        topic_name: "test_topic".to_string(),
                        config: metadata_struct::mqtt::bridge::ConnectorConfig::LocalFile(
                            metadata_struct::mqtt::bridge::config_local_file::LocalFileConnectorConfig::default(),
                        ),
                        failure_strategy: FailureHandlingStrategy::Discard,
                        status: MQTTStatus::Idle,
                        broker_id: Some(broker_id),
                        create_time: now_second(),
                        update_time: now_second(),
                    },
                );
            }
        }

        (cache_manager, cluster_name)
    }

    #[test]
    fn test_calculate_broker_load_normal_distribution() {
        let (cache_manager, cluster_name) = setup_test_cluster(3, vec![2, 3, 1]);

        let broker_load = calculate_broker_load_internal(&cache_manager, &cluster_name).unwrap();

        assert_eq!(broker_load.len(), 3, "Should have 3 brokers");
        assert_eq!(
            broker_load.get(&1),
            Some(&2),
            "Broker 1 should have 2 connectors"
        );
        assert_eq!(
            broker_load.get(&2),
            Some(&3),
            "Broker 2 should have 3 connectors"
        );
        assert_eq!(
            broker_load.get(&3),
            Some(&1),
            "Broker 3 should have 1 connector"
        );
    }

    #[test]
    fn test_calculate_broker_load_all_brokers_idle() {
        let (cache_manager, cluster_name) = setup_test_cluster(3, vec![0, 0, 0]);

        let broker_load = calculate_broker_load_internal(&cache_manager, &cluster_name).unwrap();

        assert_eq!(broker_load.len(), 3, "Should have 3 brokers");
        assert_eq!(broker_load.get(&1), Some(&0), "Broker 1 should be idle");
        assert_eq!(broker_load.get(&2), Some(&0), "Broker 2 should be idle");
        assert_eq!(broker_load.get(&3), Some(&0), "Broker 3 should be idle");
    }

    #[test]
    fn test_calculate_broker_load_more_brokers_than_connectors() {
        let (cache_manager, cluster_name) = setup_test_cluster(5, vec![1, 0, 2, 0, 1]);

        let broker_load = calculate_broker_load_internal(&cache_manager, &cluster_name).unwrap();

        assert_eq!(broker_load.len(), 5, "Should have 5 brokers");
        assert_eq!(
            broker_load.get(&1),
            Some(&1),
            "Broker 1 should have 1 connector"
        );
        assert_eq!(broker_load.get(&2), Some(&0), "Broker 2 should be idle");
        assert_eq!(
            broker_load.get(&3),
            Some(&2),
            "Broker 3 should have 2 connectors"
        );
        assert_eq!(broker_load.get(&4), Some(&0), "Broker 4 should be idle");
        assert_eq!(
            broker_load.get(&5),
            Some(&1),
            "Broker 5 should have 1 connector"
        );
    }

    #[test]
    fn test_calculate_broker_load_ignores_unassigned_connectors() {
        let (cache_manager, cluster_name) = setup_test_cluster(2, vec![2, 1]);

        // Add unassigned connectors
        for i in 0..3 {
            cache_manager.add_connector(
                &cluster_name,
                &MQTTConnector {
                    cluster_name: cluster_name.clone(),
                    connector_name: format!("unassigned_{}", i),
                    connector_type: ConnectorType::LocalFile,
                    topic_name: "test_topic".to_string(),
                    config: metadata_struct::mqtt::bridge::ConnectorConfig::LocalFile(
                        metadata_struct::mqtt::bridge::config_local_file::LocalFileConnectorConfig::default(),
                    ),
                    failure_strategy: FailureHandlingStrategy::Discard,
                    status: MQTTStatus::Idle,
                    broker_id: None, // Unassigned - should be ignored
                    create_time: now_second(),
                    update_time: now_second(),
                },
            );
        }

        let broker_load = calculate_broker_load_internal(&cache_manager, &cluster_name).unwrap();

        assert_eq!(broker_load.len(), 2, "Should have 2 brokers");
        assert_eq!(
            broker_load.get(&1),
            Some(&2),
            "Broker 1: only count assigned connectors"
        );
        assert_eq!(
            broker_load.get(&2),
            Some(&1),
            "Broker 2: only count assigned connectors"
        );
    }

    #[test]
    fn test_calculate_broker_load_empty_cluster_returns_error() {
        let empty_cache = Arc::new(CacheManager::new(test_rocksdb_instance()));

        let result = calculate_broker_load_internal(&empty_cache, "nonexistent_cluster");

        assert!(result.is_err(), "Should return error for empty cluster");
        assert!(
            matches!(result.unwrap_err(), MetaServiceError::NoAvailableBrokerNode),
            "Should return NoAvailableBrokerNode error"
        );
    }

    #[test]
    fn test_calculate_broker_load_single_broker_multiple_connectors() {
        let (cache_manager, cluster_name) = setup_test_cluster(1, vec![10]);

        let broker_load = calculate_broker_load_internal(&cache_manager, &cluster_name).unwrap();

        assert_eq!(broker_load.len(), 1, "Should have 1 broker");
        assert_eq!(
            broker_load.get(&1),
            Some(&10),
            "Broker 1 should have 10 connectors"
        );
    }
}
