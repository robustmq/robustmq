use crate::storage::{
    cluster::ClusterInfo, node::NodeInfo, segment::SegmentInfo, shard::ShardInfo,
};
use common::tools::now_mills;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct EngineClusterCache {
    pub cluster_list: HashMap<String, ClusterInfo>,
    pub node_list: HashMap<String, NodeInfo>,
    pub shard_list: HashMap<String, ShardInfo>,
    pub segment_list: HashMap<String, SegmentInfo>,
    pub node_heartbeat: HashMap<u64, u128>,
}

impl EngineClusterCache {
    pub fn new() -> EngineClusterCache {
        let bc = EngineClusterCache::default();
        return bc;
    }

    pub fn add_cluster(&mut self, cluster: ClusterInfo) {
        self.cluster_list
            .insert(cluster.cluster_name.clone(), cluster);
    }

    pub fn add_cluster_node(&mut self, cluster_name: String, node_id: u64) {
        if !self.cluster_list.contains_key(&cluster_name) {
            return;
        }
        let mut cluster = self.cluster_list.remove(&cluster_name).unwrap();
        if !cluster.nodes.contains(&node_id) {
            cluster.nodes.push(node_id);
        }
        self.add_cluster(cluster);
    }

    pub fn remove_cluster_node(&mut self, cluster_name: String, node_id: u64) {
        if !self.cluster_list.contains_key(&cluster_name) {
            return;
        }
        let mut cluster = self.cluster_list.remove(&cluster_name).unwrap();
        match cluster.nodes.binary_search(&node_id) {
            Ok(index) => {
                cluster.nodes.remove(index);
                self.add_cluster(cluster);
            }
            Err(_) => {
                self.add_cluster(cluster);
            }
        }
    }

    pub fn add_node(&mut self, node: NodeInfo) {
        self.node_list.insert(
            self.node_key(node.cluster_name.clone(), node.node_id),
            node.clone(),
        );

        self.heart_time(node.node_id, now_mills());
    }

    pub fn remove_node(&mut self, cluster_name: String, node_id: u64) {
        self.node_list.remove(&self.node_key(cluster_name, node_id));
        self.node_heartbeat.remove(&node_id);
    }

    pub fn add_shard(&mut self, shard: ShardInfo) {
        self.shard_list.insert(
            self.shard_key(shard.cluster_name.clone(), shard.shard_name.clone()),
            shard,
        );
    }

    pub fn remove_shard(&mut self, cluster_name: String, shard_name: String) {
        self.shard_list
            .remove(&self.shard_key(cluster_name, shard_name));
    }

    pub fn next_segment_seq(&self, cluster_name: &String, shard_name: &String) -> u64 {
        if let Some(shard) = self.get_shard(cluster_name.clone(), shard_name.clone()) {
            return shard.last_segment_seq + 1;
        }
        return 1;
    }

    pub fn get_shard(&self, cluster_name: String, shard_name: String) -> Option<ShardInfo> {
        let key = self.shard_key(cluster_name, shard_name);
        if let Some(shard) = self.shard_list.get(&key) {
            return Some(shard.clone());
        }
        return None;
    }

    pub fn add_segment(&mut self, segment: SegmentInfo) {
        let key = self.segment_key(
            segment.cluster_name.clone(),
            segment.shard_name.clone(),
            segment.segment_seq,
        );

        self.segment_list.insert(key, segment.clone());

        if let Some(mut shard) =
            self.get_shard(segment.cluster_name.clone(), segment.shard_name.clone())
        {
            if !shard.segments.contains(&segment.segment_seq) {
                shard.segments.push(segment.segment_seq);
                shard.last_segment_seq = segment.segment_seq;
                self.add_shard(shard);
            }
        }
    }

    pub fn remove_segment(&mut self, cluster_name: String, shard_name: String, segment_seq: u64) {
        let key = self.segment_key(cluster_name.clone(), shard_name.clone(), segment_seq);
        self.segment_list.remove(&key);

        if let Some(mut shard) = self.get_shard(cluster_name.clone(), shard_name.clone()) {
            match shard.segments.binary_search(&segment_seq) {
                Ok(index) => {
                    shard.segments.remove(index);
                    self.add_shard(shard);
                }
                Err(_) => {}
            }
        }
    }

    pub fn heart_time(&mut self, node_id: u64, time: u128) {
        self.node_heartbeat.insert(node_id, time);
    }

    fn node_key(&self, cluster_name: String, node_id: u64) -> String {
        return format!("{}_{}", cluster_name, node_id);
    }

    fn shard_key(&self, cluster_name: String, shard_name: String) -> String {
        return format!("{}_{}", cluster_name, shard_name);
    }

    fn segment_key(&self, cluster_name: String, shard_name: String, segment_num: u64) -> String {
        return format!("{}_{}_{}", cluster_name, shard_name, segment_num);
    }
}
