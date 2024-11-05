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

use log::info;
use preferred_election::PreferredElection;

pub mod call_node;
pub mod gc;
pub mod preferred_election;

pub struct StorageEngineController {}

impl StorageEngineController {
    pub fn new() -> StorageEngineController {
        let controller = StorageEngineController {};
        controller.load_cache();
        controller
    }

    pub async fn start(&self) {
        self.resource_manager_thread();
        self.preferred_replica_election();
        info!("Storage Engine Controller started successfully");
    }

    pub fn load_cache(&self) {
        // let cluster_handler = ClusterStorage::new(self.rocksdb_engine_handler.clone());
        // let cluster_list =
        //     cluster_handler.list(Some(ClusterType::JournalServer.as_str_name().to_string()));

        // let mut engine_cache = self.engine_cache.write().unwrap();
        // let node_handler = NodeStorage::new(self.rocksdb_engine_handler.clone());
        // let shard_handler = ShardStorage::new(self.rocksdb_engine_handler.clone());

        // for cluster in cluster_list {
        //     let cluster_name = cluster.cluster_name.clone();

        //     // load shard cache
        //     let shard_list = shard_handler.shard_list(cluster_name.clone());
        //     for shard in shard_list {
        //         engine_cache.add_shard(shard.clone());
        //         let segment_list =
        //             shard_handler.segment_list(cluster_name.clone(), shard.shard_name);
        //         for segment in segment_list {
        //             engine_cache.add_segment(segment);
        //         }
        //     }
        // }
    }

    pub fn resource_manager_thread(&self) {
        tokio::spawn(async move {});
    }

    pub fn preferred_replica_election(&self) {
        let election = PreferredElection::new();
        tokio::spawn(async move {
            election.start().await;
        });
    }
}
