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

use std::sync::Arc;
use std::time::Duration;

use axum::async_trait;
use bson::Document;
use grpc_clients::pool::ClientPool;
use metadata_struct::{
    mqtt::bridge::config_mongodb::MongoDBConnectorConfig, mqtt::bridge::connector::MQTTConnector,
    storage::adapter_record::AdapterWriteRecord,
};
use mongodb::{
    options::{ClientOptions, InsertManyOptions, WriteConcern},
    Client, Collection,
};
use storage_adapter::driver::StorageDriverManager;
use tracing::{debug, error, info, warn};

use crate::handler::error::MqttBrokerError;
use crate::handler::tool::ResultMqttBrokerError;

use super::{
    core::{run_connector_loop, BridgePluginReadConfig, BridgePluginThread, ConnectorSink},
    manager::ConnectorManager,
};

pub struct MongoDBBridgePlugin {
    config: MongoDBConnectorConfig,
}

impl MongoDBBridgePlugin {
    pub fn new(config: MongoDBConnectorConfig) -> Self {
        MongoDBBridgePlugin { config }
    }

    async fn create_client(&self) -> Result<Client, MqttBrokerError> {
        let uri = self.config.build_connection_uri();
        debug!("Connecting to MongoDB at {}", self.config.host);

        let mut client_options = ClientOptions::parse(&uri).await.map_err(|e| {
            MqttBrokerError::MongoDBError(format!(
                "Failed to parse MongoDB URI for {}: {}",
                self.config.host, e
            ))
        })?;

        client_options.connect_timeout =
            Some(Duration::from_secs(self.config.connect_timeout_secs));
        client_options.server_selection_timeout = Some(Duration::from_secs(
            self.config.server_selection_timeout_secs,
        ));

        if let Some(max_pool) = self.config.max_pool_size {
            client_options.max_pool_size = Some(max_pool);
        }
        if let Some(min_pool) = self.config.min_pool_size {
            client_options.min_pool_size = Some(min_pool);
        }

        let w_value = if self.config.w == "majority" {
            WriteConcern::builder()
                .w(mongodb::options::Acknowledgment::Majority)
                .build()
        } else if let Ok(w_num) = self.config.w.parse::<u32>() {
            WriteConcern::builder()
                .w(mongodb::options::Acknowledgment::Nodes(w_num))
                .build()
        } else {
            WriteConcern::builder()
                .w(mongodb::options::Acknowledgment::Nodes(1))
                .build()
        };
        client_options.write_concern = Some(w_value);

        Client::with_options(client_options).map_err(|e| {
            MqttBrokerError::MongoDBError(format!(
                "Failed to create MongoDB client for {}:{}: {}",
                self.config.host, self.config.port, e
            ))
        })
    }

    fn record_to_document(&self, record: &AdapterWriteRecord) -> Result<Document, MqttBrokerError> {
        bson::to_document(record).map_err(|e| {
            MqttBrokerError::BsonSerializationError(format!(
                "Failed to serialize record with key '{:?}' at timestamp {}: {}",
                record.key, record.timestamp, e
            ))
        })
    }
}

#[async_trait]
impl ConnectorSink for MongoDBBridgePlugin {
    type SinkResource = Collection<Document>;

    async fn validate(&self) -> ResultMqttBrokerError {
        info!(
            "Validating MongoDB connector configuration for {}:{}",
            self.config.host, self.config.port
        );

        let client = self.create_client().await?;

        let db = client.database(&self.config.database);

        match tokio::time::timeout(
            Duration::from_secs(self.config.server_selection_timeout_secs),
            db.run_command(bson::doc! { "ping": 1 }, None),
        )
        .await
        {
            Ok(Ok(_)) => {
                info!(
                    "Successfully validated MongoDB connection to {}:{}/{}",
                    self.config.host, self.config.port, self.config.database
                );
                Ok(())
            }
            Ok(Err(e)) => Err(MqttBrokerError::MongoDBError(format!(
                "MongoDB ping failed for {}:{}/{}: {}",
                self.config.host, self.config.port, self.config.database, e
            ))),
            Err(_) => Err(MqttBrokerError::MongoDBError(format!(
                "MongoDB ping timeout for {}:{}/{}",
                self.config.host, self.config.port, self.config.database
            ))),
        }
    }

    async fn init_sink(&self) -> Result<Self::SinkResource, MqttBrokerError> {
        info!(
            "Initializing MongoDB sink: {}:{}/{}/{}",
            self.config.host, self.config.port, self.config.database, self.config.collection
        );
        let client = self.create_client().await?;
        let db = client.database(&self.config.database);
        let collection = db.collection(&self.config.collection);
        info!("MongoDB sink initialized successfully");
        Ok(collection)
    }

    async fn send_batch(
        &self,
        records: &[AdapterWriteRecord],
        collection: &mut Collection<Document>,
    ) -> ResultMqttBrokerError {
        if records.is_empty() {
            return Ok(());
        }

        debug!("Processing batch of {} records for MongoDB", records.len());

        let mut documents = Vec::with_capacity(records.len());
        let mut failed_serializations = Vec::new();

        for (idx, record) in records.iter().enumerate() {
            match self.record_to_document(record) {
                Ok(doc) => documents.push(doc),
                Err(e) => {
                    warn!(
                        "Failed to serialize record {}/{} (key: '{:?}', timestamp: {}): {}",
                        idx + 1,
                        records.len(),
                        record.key,
                        record.timestamp,
                        e
                    );
                    failed_serializations.push((idx, e));
                }
            }
        }

        if documents.is_empty() {
            return Err(MqttBrokerError::MongoDBError(format!(
                "All {} records failed to serialize to BSON",
                records.len()
            )));
        }

        if !failed_serializations.is_empty() {
            warn!(
                "{}/{} records failed serialization and will not be inserted",
                failed_serializations.len(),
                records.len()
            );
        }

        let options = InsertManyOptions::builder()
            .ordered(self.config.ordered_insert)
            .build();

        debug!(
            "Inserting {} documents into MongoDB (ordered={})",
            documents.len(),
            self.config.ordered_insert
        );

        match collection.insert_many(documents, options).await {
            Ok(result) => {
                info!(
                    "Successfully inserted {} documents into {}.{}",
                    result.inserted_ids.len(),
                    self.config.database,
                    self.config.collection
                );
                Ok(())
            }
            Err(e) => {
                let error_msg = format!(
                    "Failed to insert documents into {}.{}: {}",
                    self.config.database, self.config.collection, e
                );
                error!("{}", error_msg);
                Err(MqttBrokerError::MongoDBError(error_msg))
            }
        }
    }
}

pub fn start_mongodb_connector(
    client_pool: Arc<ClientPool>,
    connector_manager: Arc<ConnectorManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    connector: MQTTConnector,
    thread: BridgePluginThread,
) {
    tokio::spawn(Box::pin(async move {
        let mongodb_config = match &connector.config {
            metadata_struct::mqtt::bridge::ConnectorConfig::MongoDB(config) => config.clone(),
            _ => {
                error!("Invalid connector config type, expected MongoDB config");
                return;
            }
        };

        let batch_size = mongodb_config.batch_size as u64;
        let bridge = MongoDBBridgePlugin::new(mongodb_config);

        let stop_recv = thread.stop_send.subscribe();
        connector_manager.add_connector_thread(&connector.connector_name, thread);

        if let Err(e) = run_connector_loop(
            &bridge,
            &client_pool,
            &connector_manager,
            storage_driver_manager.clone(),
            connector.connector_name.clone(),
            BridgePluginReadConfig {
                topic_name: connector.topic_name,
                record_num: batch_size,
                strategy: connector.failure_strategy,
            },
            stop_recv,
        )
        .await
        {
            connector_manager.remove_connector_thread(&connector.connector_name);
            error!(
                "Failed to start MongoDBBridgePlugin with error message: {:?}",
                e
            );
        }
    }));
}
