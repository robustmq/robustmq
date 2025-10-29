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

use std::{sync::Arc, time::Duration};

use axum::async_trait;
use bson::Document;
use metadata_struct::{
    adapter::record::Record, mqtt::bridge::config_mongodb::MongoDBConnectorConfig,
    mqtt::bridge::connector::MQTTConnector,
};
use mongodb::{options::ClientOptions, Client, Collection};
use storage_adapter::storage::ArcStorageAdapter;
use tokio::{select, sync::broadcast, time::sleep};
use tracing::{error, info};

use crate::common::types::ResultMqttBrokerError;
use crate::handler::error::MqttBrokerError;
use crate::storage::message::MessageStorage;

use super::{
    core::{BridgePlugin, BridgePluginReadConfig, BridgePluginThread},
    manager::ConnectorManager,
};

pub struct MongoDBBridgePlugin {
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector_name: String,
    config: MongoDBConnectorConfig,
    stop_send: broadcast::Sender<bool>,
}

impl MongoDBBridgePlugin {
    pub fn new(
        connector_manager: Arc<ConnectorManager>,
        message_storage: ArcStorageAdapter,
        connector_name: String,
        config: MongoDBConnectorConfig,
        stop_send: broadcast::Sender<bool>,
    ) -> Self {
        MongoDBBridgePlugin {
            connector_manager,
            message_storage,
            connector_name,
            config,
            stop_send,
        }
    }

    /// Create MongoDB client with connection pool
    async fn create_client(&self) -> Result<Client, MqttBrokerError> {
        let uri = self.config.build_connection_uri();
        let client_options = ClientOptions::parse(&uri)
            .await
            .map_err(|e| MqttBrokerError::MongoDBError(e.to_string()))?;

        Client::with_options(client_options)
            .map_err(|e| MqttBrokerError::MongoDBError(e.to_string()))
    }

    /// Convert Record to MongoDB Document
    fn record_to_document(&self, record: &Record) -> Result<Document, MqttBrokerError> {
        // Serialize the entire record to BSON document
        bson::to_document(record)
            .map_err(|e| MqttBrokerError::BsonSerializationError(e.to_string()))
    }

    /// Batch insert records to MongoDB
    pub async fn append(
        &self,
        records: &Vec<Record>,
        collection: &Collection<Document>,
    ) -> ResultMqttBrokerError {
        if records.is_empty() {
            return Ok(());
        }

        // Convert records to documents
        let mut documents = Vec::with_capacity(records.len());
        for record in records {
            match self.record_to_document(record) {
                Ok(doc) => documents.push(doc),
                Err(e) => {
                    error!(
                        "Failed to convert record to MongoDB document: {}. Record will be skipped.",
                        e
                    );
                    continue;
                }
            }
        }

        if documents.is_empty() {
            return Ok(());
        }

        // Batch insert
        match collection.insert_many(documents, None).await {
            Ok(result) => {
                info!(
                    "Successfully inserted {} documents into MongoDB collection '{}'",
                    result.inserted_ids.len(),
                    self.config.collection
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    "Failed to insert documents into MongoDB collection '{}': {}",
                    self.config.collection, e
                );
                Err(MqttBrokerError::MongoDBError(e.to_string()))
            }
        }
    }
}

pub fn start_mongodb_connector(
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector: MQTTConnector,
    thread: BridgePluginThread,
) {
    tokio::spawn(async move {
        let mongodb_config = match serde_json::from_str::<MongoDBConnectorConfig>(&connector.config)
        {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to parse MongoDBConnectorConfig with error message: {}, configuration contents: {}", e, connector.config);
                return;
            }
        };

        let bridge = MongoDBBridgePlugin::new(
            connector_manager.clone(),
            message_storage.clone(),
            connector.connector_name.clone(),
            mongodb_config,
            thread.stop_send.clone(),
        );

        connector_manager.add_connector_thread(&connector.connector_name, thread);

        if let Err(e) = bridge
            .exec(BridgePluginReadConfig {
                topic_name: connector.topic_name,
                record_num: 100,
            })
            .await
        {
            connector_manager.remove_connector_thread(&connector.connector_name);
            error!(
                "Failed to start MongoDBBridgePlugin with error message: {:?}",
                e
            );
        }
    });
}

#[async_trait]
impl BridgePlugin for MongoDBBridgePlugin {
    async fn exec(&self, config: BridgePluginReadConfig) -> ResultMqttBrokerError {
        let message_storage = MessageStorage::new(self.message_storage.clone());
        let group_name = self.connector_name.clone();
        let mut recv = self.stop_send.subscribe();

        // Create MongoDB client
        let client = match self.create_client().await {
            Ok(client) => {
                info!(
                    "Successfully connected to MongoDB at {}:{}",
                    self.config.host, self.config.port
                );
                client
            }
            Err(e) => {
                error!(
                    "Failed to connect to MongoDB at {}:{}: {}",
                    self.config.host, self.config.port, e
                );
                return Err(e);
            }
        };

        // Get collection reference
        let db = client.database(&self.config.database);
        let collection: Collection<Document> = db.collection(&self.config.collection);

        loop {
            let offset = message_storage.get_group_offset(&group_name).await?;

            select! {
                val = recv.recv() => {
                    if let Ok(flag) = val {
                        if flag {
                            info!(
                                "MongoDB connector '{}' thread exited successfully",
                                self.connector_name
                            );
                            break;
                        }
                    }
                }

                val = message_storage.read_topic_message(&config.topic_name, offset, config.record_num) => {
                    match val {
                        Ok(data) => {
                            self.connector_manager.report_heartbeat(&self.connector_name);

                            if data.is_empty() {
                                sleep(Duration::from_millis(100)).await;
                                continue;
                            }

                            if let Err(e) = self.append(&data, &collection).await {
                                error!(
                                    "Connector '{}' failed to write data to MongoDB collection '{}', error: {}",
                                    self.connector_name,
                                    self.config.collection,
                                    e
                                );
                                sleep(Duration::from_millis(100)).await;
                            }
                        }
                        Err(e) => {
                            error!(
                                "Connector '{}' failed to read Topic '{}' data with error: {}",
                                self.connector_name,
                                config.topic_name,
                                e
                            );
                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
