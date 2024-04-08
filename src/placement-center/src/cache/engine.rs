use crate::storage::{ segment::SegmentInfo, shard::ShardInfo,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct EngineCache {
    pub shard_list: HashMap<String, ShardInfo>,
    pub segment_list: HashMap<String, SegmentInfo>,
}

impl EngineCache {
    pub fn new() -> EngineCache {
        let bc = EngineCache::default();
        return bc;
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

    fn shard_key(&self, cluster_name: String, shard_name: String) -> String {
        return format!("{}_{}", cluster_name, shard_name);
    }

    fn segment_key(&self, cluster_name: String, shard_name: String, segment_num: u64) -> String {
        return format!("{}_{}_{}", cluster_name, shard_name, segment_num);
    }
}
