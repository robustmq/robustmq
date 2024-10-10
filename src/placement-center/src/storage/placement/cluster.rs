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

use common_base::error::common::CommonError;
use metadata_struct::placement::cluster::ClusterInfo;

use crate::storage::engine::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
    engine_save_by_cluster,
};
use crate::storage::keys::{key_cluster, key_cluster_prefix, key_cluster_prefix_by_type};
use crate::storage::rocksdb::RocksDBEngine;

pub struct ClusterStorage {
    /// The RocksDB engine handler
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ClusterStorage {
    /// Create a new ClusterStorge instance.
    ///
    /// Parameters:
    /// - `rocksdb_engine_handler: Arc<RocksDBEngine>`: The RocksDB engine handler.
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        ClusterStorage {
            rocksdb_engine_handler,
        }
    }

    /// Save cluster information to RocksDB.
    ///
    /// Parameters:
    /// - `cluster_name: String`: The name of the cluster.
    /// - `cluster_type: String`: The type of the cluster.
    ///
    /// Returns:
    /// - `OK()`: Indicates that the cluster information has been successfully saved.
    /// - `Err (CommonError)`: Indicates that the operation failed, and CommonError is the error type that includes the reason for the failure.
    pub fn save(&self, cluster_info: &ClusterInfo) -> Result<(), CommonError> {
        let key = key_cluster(&cluster_info.cluster_type, &cluster_info.cluster_name);
        engine_save_by_cluster(
            self.rocksdb_engine_handler.clone(),
            key,
            cluster_info.clone(),
        )
    }

    ///Retrieve cluster information from RocksDB.
    ///
    /// Parameters:
    /// - `cluster_name: String`: The name of the cluster.
    /// - `cluster_type: String`: The type of the cluster.
    ///
    ///Returns:
    /// - `OK(Some(ClusterInfo))`: If the cluster exists.
    /// - `OK(None)`: If the cluster does not exist.
    /// - `Err (CommonError)`: Indicates that the operation failed, and CommonError is the error type that includes the reason for the failure.
    #[allow(dead_code)]
    pub fn get(
        &self,
        cluster_type: &str,
        cluster_name: &str,
    ) -> Result<Option<ClusterInfo>, CommonError> {
        let key = key_cluster(cluster_type, cluster_name);
        match engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key) {
            Ok(Some(data)) => match serde_json::from_slice::<ClusterInfo>(&data.data) {
                Ok(info) => Ok(Some(info)),
                Err(e) => Err(e.into()),
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Delete cluster information from RocksDB.
    ///
    /// Parameters:
    /// - `cluster_name: String`: The name of the cluster.
    /// - `cluster_type: String`: The type of the cluster.
    ///
    /// Returns:
    /// - `OK()`: Indicates that the cluster has been successfully deleted.
    /// - `Err (CommonError)`: Indicates that the operation failed, and CommonError is the error type that includes the reason for the failure.
    #[allow(dead_code)]
    pub fn delete(&self, cluster_type: &str, cluster_name: &str) -> Result<(), CommonError> {
        let key: String = key_cluster(cluster_type, cluster_name);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)
    }

    ///List cluster information from RocksDB.
    ///
    /// Parameters:
    /// - `cluster_type: String`(Option): The type of the cluster.
    ///
    ///Returns:
    /// - `OK(Vec<ClusterInfo>)`: Containing the list of cluster information.
    /// - `Err (CommonError)`: Indicates that the operation failed, and CommonError is the error type that includes the reason for the failure.
    pub fn list(&self, cluster_type: Option<String>) -> Result<Vec<ClusterInfo>, CommonError> {
        let prefix_key = if let Some(ct) = cluster_type {
            key_cluster_prefix_by_type(&ct)
        } else {
            key_cluster_prefix()
        };
        match engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key) {
            Ok(data) => {
                let mut results = Vec::new();
                for raw in data {
                    match serde_json::from_slice::<ClusterInfo>(&raw.data) {
                        Ok(topic) => {
                            results.push(topic);
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                Ok(results)
            }
            Err(e) => Err(e),
        }
    }
}
