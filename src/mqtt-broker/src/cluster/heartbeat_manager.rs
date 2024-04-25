use crate::server::MQTTProtocol;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct HeartbeatManager {
    shard_num: u64,
    pub shard_data: DashMap<u64, HeartbeatShard>,
}

impl HeartbeatManager {
    pub fn new(shard_num: u64) -> Self {
        return HeartbeatManager {
            shard_num,
            shard_data: DashMap::with_capacity(256),
        };
    }

    pub fn report_hearbeat(&self, connect_id: u64, live_time: ConnectionLiveTime) {
        let hash_num = self.calc_shard_hash_num(connect_id);
        if let Some(row) = self.shard_data.get(&hash_num) {
            row.report_hearbeat(connect_id, live_time);
        } else {
            let row = HeartbeatShard::new();
            row.report_hearbeat(connect_id, live_time);
            self.shard_data.insert(connect_id, row);
        }
    }

    pub fn remove_connect(&self, connect_id: u64) {
        let hash_num = self.calc_shard_hash_num(connect_id);
        if let Some(row) = self.shard_data.get(&hash_num) {
            row.remove_connect(connect_id);
        }
    }

    pub fn get_shard_data(&self, seq: u64) -> HeartbeatShard {
        if let Some(data) = self.shard_data.get(&seq) {
            return data.clone();
        }
        return HeartbeatShard::new();
    }

    pub fn calc_shard_hash_num(&self, connect_id: u64) -> u64 {
        return connect_id % self.shard_num;
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HeartbeatShard {
    pub heartbeat_data: DashMap<u64, ConnectionLiveTime>,
}

impl HeartbeatShard {
    pub fn new() -> Self {
        return HeartbeatShard {
            heartbeat_data: DashMap::with_capacity(256),
        };
    }

    pub fn report_hearbeat(&self, connect_id: u64, live_time: ConnectionLiveTime) {
        self.heartbeat_data.insert(connect_id, live_time);
    }

    pub fn remove_connect(&self, connect_id: u64) {
        self.heartbeat_data.remove(&connect_id);
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConnectionLiveTime {
    pub protobol: MQTTProtocol,
    pub keep_live: u16,
    pub heartbeat: u64,
}
