use std::{collections::HashMap, time::Duration};

use common_base::{log::error, tools::now_mills};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

#[derive(Clone, Serialize, Deserialize)]
pub struct HeartbeatManager {
    pub shard_num: u64,
    pub heartbeat_data: HashMap<u64, HeartbeatShard>,
}

impl HeartbeatManager {
    pub fn new(shard_num: u64) -> Self {
        return HeartbeatManager {
            shard_num,
            heartbeat_data: HashMap::new(),
        };
    }

    // TCP connection heartbeat detection is performed in parallel, and subsequent processing is carried out
    pub async fn start_heartbeat_check(&self) {
        loop {
            let mut heartbeat_data = self.heartbeat_data.clone();
            let mut handles = Vec::new();
            for i in 0..self.shard_num {
                let data = heartbeat_data.remove(&i);
                handles.push(tokio::spawn(async move {
                    if let Some(da) = data {
                        for (connect_id, time) in da.heartbeat_data {
                            if (now_mills() - time.heartbeat) > (time.keep_live as u128) {
                                //todo close connect
                            }
                        }
                    }
                }));
            }
            // Waiting for all spawn to complete, thinking about the next batch of detection
            for handle in handles {
                match handle.await {
                    Ok(_) => {}
                    Err(e) => {
                        error(e.to_string());
                    }
                }
            }

            sleep(Duration::from_secs(1)).await;
        }
    }

    pub fn report_hearbeat(&mut self, connect_id: u64, live_time: ConnectionLiveTime) {
        let hash_num = self.calc_shard_hash_num(connect_id);
        if let Some(mut row) = self.heartbeat_data.remove(&hash_num) {
            row.report_hearbeat(connect_id, live_time);
            self.heartbeat_data.insert(connect_id, row);
        }
    }

    pub fn remove_connect(&mut self, connect_id: u64) {
        let hash_num = self.calc_shard_hash_num(connect_id);
        if let Some(mut row) = self.heartbeat_data.remove(&hash_num) {
            row.remove_connect(connect_id);
            self.heartbeat_data.insert(connect_id, row);
        }
    }

    pub fn calc_shard_hash_num(&self, connect_id: u64) -> u64 {
        return connect_id % self.shard_num;
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HeartbeatShard {
    pub heartbeat_data: HashMap<u64, ConnectionLiveTime>,
}

impl HeartbeatShard {
    pub fn new() -> Self {
        return HeartbeatShard {
            heartbeat_data: HashMap::new(),
        };
    }

    pub fn report_hearbeat(&mut self, connect_id: u64, live_time: ConnectionLiveTime) {
        self.heartbeat_data.insert(connect_id, live_time);
    }

    pub fn remove_connect(&mut self, connect_id: u64) {
        self.heartbeat_data.remove(&connect_id);
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConnectionLiveTime {
    pub keep_live: u16,
    pub heartbeat: u128,
}
