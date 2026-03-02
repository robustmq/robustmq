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

use crate::{
    controller::connector_status::ConnectorStatus,
    core::{cache::MetaCacheManager, error::MetaServiceError},
    raft::manager::MultiRaftManager,
};
use common_base::{
    error::ResultCommonError,
    tools::{loop_select_ticket, now_second},
};
use common_config::broker::broker_config;
use metadata_struct::mqtt::bridge::{connector::MQTTConnector, status::MQTTStatus};
use node_call::NodeCallManager;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;
use tracing::{info, warn};

pub struct ConnectorScheduler {
    cache_manager: Arc<MetaCacheManager>,
    heartbeat_timeout_sec: u64,
    connector_context: ConnectorStatus,
}

impl ConnectorScheduler {
    pub fn new(
        raft_manager: Arc<MultiRaftManager>,
        node_call_manager: Arc<NodeCallManager>,
        cache_manager: Arc<MetaCacheManager>,
    ) -> Self {
        let config = broker_config();
        let connector_context =
            ConnectorStatus::new(raft_manager, node_call_manager, cache_manager.clone());
        Self {
            cache_manager,
            heartbeat_timeout_sec: config.meta_runtime.heartbeat_timeout_ms / 1000,
            connector_context,
        }
    }

    pub async fn run(&self, stop_send: &broadcast::Sender<bool>) {
        let ac_fn = async move || -> ResultCommonError {
            if let Err(e) = self.check_heartbeat().await {
                warn!("Heartbeat check failed: {:?}", e);
            }

            if let Err(e) = self.start_stop_connector_thread().await {
                warn!("Connector scheduling failed: {:?}", e);
            }
            Ok(())
        };

        loop_select_ticket(ac_fn, 1000, stop_send).await;
    }
}

impl ConnectorScheduler {
    async fn check_heartbeat(&self) -> Result<(), MetaServiceError> {
        let current_time = now_second();

        for heartbeat in self.cache_manager.get_all_connector_heartbeat() {
            if !self
                .cache_manager
                .connector_list
                .contains_key(&heartbeat.connector_name)
            {
                self.cache_manager
                    .remove_connector_heartbeat(&heartbeat.connector_name);
                continue;
            }

            let elapsed_time = current_time - heartbeat.last_heartbeat;
            if elapsed_time > self.heartbeat_timeout_sec {
                warn!(
                    "Connector heartbeat expired: name={}, elapsed={}s, timeout={}s",
                    heartbeat.connector_name, elapsed_time, self.heartbeat_timeout_sec
                );
                if let Err(e) = self
                    .connector_context
                    .update_status_to_idle(&heartbeat.connector_name)
                    .await
                {
                    warn!(
                        "Failed to reset connector {} to Idle: {:?}",
                        heartbeat.connector_name, e
                    );
                }
            }
        }

        Ok(())
    }

    async fn start_stop_connector_thread(&self) -> Result<(), MetaServiceError> {
        let mut idle_connectors = Vec::new();

        for connector in self.cache_manager.get_all_connector() {
            if connector.broker_id.is_none() && connector.status == MQTTStatus::Running {
                let _ = self
                    .connector_context
                    .update_status_to_idle(&connector.connector_name)
                    .await;
                continue;
            }

            if connector.status == MQTTStatus::Idle {
                idle_connectors.push(connector);
            }
        }

        self.assign_and_start(&idle_connectors).await
    }

    async fn assign_and_start(
        &self,
        idle_connectors: &[MQTTConnector],
    ) -> Result<(), MetaServiceError> {
        if idle_connectors.is_empty() {
            return Ok(());
        }

        let mut broker_load = calculate_broker_load_internal(&self.cache_manager)?;

        for connector in idle_connectors {
            let mut connector = connector.clone();

            if connector.broker_id.is_none() {
                let broker_id = match broker_load
                    .iter()
                    .min_by_key(|(_, count)| *count)
                    .map(|(id, _)| *id)
                {
                    Some(id) => id,
                    None => {
                        warn!(
                            "No available broker for connector {}",
                            connector.connector_name
                        );
                        continue;
                    }
                };

                connector.broker_id = Some(broker_id);
                *broker_load.entry(broker_id).or_insert(0) += 1;

                info!(
                    "Connector {} assigned to Broker {} (load: {})",
                    connector.connector_name, broker_id, broker_load[&broker_id]
                );
            }

            connector.status = MQTTStatus::Running;

            if let Err(e) = self
                .connector_context
                .save_connector(connector.clone())
                .await
            {
                warn!(
                    "Failed to save connector {}: {:?}",
                    connector.connector_name, e
                );
                if let Some(bid) = connector.broker_id {
                    if let Some(count) = broker_load.get_mut(&bid) {
                        *count = count.saturating_sub(1);
                    }
                }
            }
        }

        Ok(())
    }
}

fn calculate_broker_load_internal(
    cache_manager: &MetaCacheManager,
) -> Result<HashMap<u64, usize>, MetaServiceError> {
    let mut broker_load: HashMap<u64, usize> = cache_manager
        .node_list
        .iter()
        .map(|node| (node.node_id, 0))
        .collect();

    if broker_load.is_empty() {
        return Err(MetaServiceError::NoAvailableBrokerNode);
    }

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
    use metadata_struct::mqtt::node_extend::NodeExtend;
    use rocksdb_engine::test::test_rocksdb_instance;

    fn setup_test_cluster(
        broker_count: usize,
        connector_distribution: Vec<usize>,
    ) -> Arc<MetaCacheManager> {
        let cache_manager = Arc::new(MetaCacheManager::new(test_rocksdb_instance()));

        for i in 1..=broker_count {
            cache_manager.add_broker_node(BrokerNode {
                node_id: i as u64,
                node_ip: format!("127.0.0.{}", i),
                grpc_addr: format!("127.0.0.{}:9000", i),
                extend: NodeExtend::default(),
                roles: Vec::new(),
                start_time: now_second(),
                register_time: now_second(),
                storage_fold: Vec::new(),
                ..Default::default()
            });
        }

        for (broker_idx, &count) in connector_distribution.iter().enumerate() {
            let broker_id = (broker_idx + 1) as u64;
            for j in 0..count {
                cache_manager.add_connector(MQTTConnector {
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
                });
            }
        }

        cache_manager
    }

    fn make_unassigned_connector(name: &str) -> MQTTConnector {
        MQTTConnector {
            connector_name: name.to_string(),
            connector_type: ConnectorType::LocalFile,
            topic_name: "test_topic".to_string(),
            config: metadata_struct::mqtt::bridge::ConnectorConfig::LocalFile(
                metadata_struct::mqtt::bridge::config_local_file::LocalFileConnectorConfig::default(
                ),
            ),
            failure_strategy: FailureHandlingStrategy::Discard,
            status: MQTTStatus::Idle,
            broker_id: None,
            create_time: now_second(),
            update_time: now_second(),
        }
    }

    #[test]
    fn test_calculate_broker_load() {
        // empty cluster returns error
        let empty = Arc::new(MetaCacheManager::new(test_rocksdb_instance()));
        assert!(matches!(
            calculate_broker_load_internal(&empty).unwrap_err(),
            MetaServiceError::NoAvailableBrokerNode
        ));

        // normal distribution
        let cm = setup_test_cluster(3, vec![2, 3, 1]);
        let load = calculate_broker_load_internal(&cm).unwrap();
        assert_eq!(load[&1], 2);
        assert_eq!(load[&2], 3);
        assert_eq!(load[&3], 1);

        // all idle
        let cm = setup_test_cluster(3, vec![0, 0, 0]);
        let load = calculate_broker_load_internal(&cm).unwrap();
        assert!(load.values().all(|&v| v == 0));
        assert_eq!(load.len(), 3);

        // unassigned connectors are ignored
        let cm = setup_test_cluster(2, vec![2, 1]);
        for i in 0..3 {
            cm.add_connector(make_unassigned_connector(&format!("unassigned_{}", i)));
        }
        let load = calculate_broker_load_internal(&cm).unwrap();
        assert_eq!(load[&1], 2);
        assert_eq!(load[&2], 1);
    }
}
