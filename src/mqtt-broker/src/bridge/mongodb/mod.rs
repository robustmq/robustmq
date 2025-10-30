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

use axum::async_trait;
use bson::Document;
use metadata_struct::{
    adapter::record::Record, mqtt::bridge::config_mongodb::MongoDBConnectorConfig,
    mqtt::bridge::connector::MQTTConnector,
};
use mongodb::{options::ClientOptions, Client, Collection};
use storage_adapter::storage::ArcStorageAdapter;
use tracing::error;

use crate::common::types::ResultMqttBrokerError;
use crate::handler::error::MqttBrokerError;

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
        let client_options = ClientOptions::parse(&uri)
            .await
            .map_err(|e| MqttBrokerError::MongoDBError(e.to_string()))?;

        Client::with_options(client_options)
            .map_err(|e| MqttBrokerError::MongoDBError(e.to_string()))
    }

    fn record_to_document(&self, record: &Record) -> Result<Document, MqttBrokerError> {
        bson::to_document(record)
            .map_err(|e| MqttBrokerError::BsonSerializationError(e.to_string()))
    }
}

#[async_trait]
impl ConnectorSink for MongoDBBridgePlugin {
    type SinkResource = Collection<Document>;

    async fn validate(&self) -> ResultMqttBrokerError {
        Ok(())
    }

    async fn init_sink(&self) -> Result<Self::SinkResource, MqttBrokerError> {
        let client = self.create_client().await?;
        let db = client.database(&self.config.database);
        let collection = db.collection(&self.config.collection);
        Ok(collection)
    }

    async fn send_batch(
        &self,
        records: &[Record],
        collection: &mut Collection<Document>,
    ) -> ResultMqttBrokerError {
        if records.is_empty() {
            return Ok(());
        }

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

        collection
            .insert_many(documents, None)
            .await
            .map(|_| ())
            .map_err(|e| MqttBrokerError::MongoDBError(e.to_string()))
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

        let bridge = MongoDBBridgePlugin::new(mongodb_config);

        let stop_recv = thread.stop_send.subscribe();
        connector_manager.add_connector_thread(&connector.connector_name, thread);

        if let Err(e) = run_connector_loop(
            &bridge,
            &connector_manager,
            message_storage.clone(),
            connector.connector_name.clone(),
            BridgePluginReadConfig {
                topic_name: connector.topic_name,
                record_num: 100,
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
    });
}
