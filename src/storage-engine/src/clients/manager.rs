use crate::clients::connection::{NodeConnection, MODULE_READ, MODULE_WRITE};
use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use common_base::tools::now_second;
use dashmap::DashMap;
use futures::SinkExt;
use protocol::storage::codec::StorageEnginePacket;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast::Receiver;
use tokio::time::sleep;
use tracing::error;

pub struct ClientConnectionManager {
    cache_manager: Arc<StorageCacheManager>,
    // (node_id, (index,NodeConnection))
    node_conns: DashMap<u64, DashMap<u32, NodeConnection>>,
    read_conn_atom: AtomicU64,
    write_conn_atom: AtomicU64,
    node_connection_num: u32,
}

impl ClientConnectionManager {
    pub fn new(cache_manager: Arc<StorageCacheManager>, node_connection_num: u32) -> Self {
        let node_conns = DashMap::with_capacity(2);
        let read_conn_atom = AtomicU64::new(0);
        let write_conn_atom = AtomicU64::new(0);
        ClientConnectionManager {
            node_conns,
            cache_manager,
            read_conn_atom,
            write_conn_atom,
            node_connection_num,
        }
    }

    pub async fn write_send(
        &self,
        node_id: u64,
        req_packet: StorageEnginePacket,
    ) -> Result<StorageEnginePacket, StorageEngineError> {
        let new_node_id = node_id as i64;
        if !self.node_conns.contains_key(&new_node_id) {
            let conn = NodeConnection::new(new_node_id, self.cache_manager.clone());
            conn.init_conn(MODULE_WRITE).await?;
            self.node_conns.insert(new_node_id, conn);
        }

        let conn = self.node_conns.get(&new_node_id).unwrap();
        conn.write_send(req_packet).await
    }

    pub async fn read_send(
        &self,
        node_id: u64,
        req_packet: StorageEnginePacket,
    ) -> Result<StorageEnginePacket, StorageEngineError> {
        let new_node_id = node_id as i64;
        if !self.node_conns.contains_key(&new_node_id) {
            let conn = NodeConnection::new(new_node_id, self.cache_manager.clone());
            conn.init_conn(MODULE_READ).await?;
            self.node_conns.insert(new_node_id, conn);
        }

        let conn = self.node_conns.get(&new_node_id).unwrap();
        conn.read_send(req_packet).await
    }

    pub fn get_inactive_conn(&self) -> Vec<(i64, String)> {
        let mut results = Vec::new();
        for node in self.node_conns.iter() {
            for conn in node.connection.iter() {
                if (now_second() - conn.last_active_time) > 600 {
                    results.push((node.node_id, conn.key().to_string()));
                }
            }
        }
        results
    }

    pub async fn close_conn_by_node(&self, node_id: i64, conn_type: &str) {
        if let Some(node) = self.node_conns.get(&node_id) {
            if let Some(mut conn) = node.connection.get_mut(conn_type) {
                if let Err(e) = conn.stream.close().await {
                    error!("{}", e);
                }
            }
            node.connection.remove(conn_type);
        }
    }

    pub async fn close(&self) {
        for node in self.node_conns.iter() {
            for mut conn in node.connection.iter_mut() {
                if let Err(e) = conn.stream.close().await {
                    error!("{}", e);
                }
            }
        }
    }
}

pub fn start_conn_gc_thread(
    cache_manager: Arc<StorageCacheManager>,
    connection_manager: Arc<ClientConnectionManager>,
    mut stop_recv: Receiver<bool>,
) {
    tokio::spawn(async move {
        loop {
            select! {
                val = stop_recv.recv() =>{
                    if let Err(flag) = val {
                        break;
                    }
                },
                _ = gc_conn(cache_manager.clone(),connection_manager.clone())=>{
                    sleep(Duration::from_secs(10)).await;
                }
            }
        }
    });
}

async fn gc_conn(
    cache_manager: Arc<StorageCacheManager>,
    connection_manager: Arc<ClientConnectionManager>,
) {
    for raw in connection_manager.get_inactive_conn() {
        // connection_manager.close_conn_by_node(raw.0, &raw.1).await;
    }
}
