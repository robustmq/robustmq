use crate::storage::cluster::ClusterStorage;
use clients::poll::ClientPool;
use common_base::log::debug;
use std::{sync::Arc, time::Duration};
use tokio::{select, sync::broadcast, time::sleep};

pub async fn report_heartbeat(client_poll: Arc<ClientPool>, stop_send: broadcast::Sender<bool>) {
    loop {
        let mut stop_recv = stop_send.subscribe();
        select! {
            val = stop_recv.recv() =>{
                match val{
                    Ok(flag) => {
                        if flag {
                            debug(format!("Heartbeat reporting thread exited successfully"));
                            break;
                        }
                    }
                    Err(_) => {}
                }
            }
            _ = report(client_poll.clone()) => {

            }
        }
    }
}

async fn report(client_poll: Arc<ClientPool>) {
    let cluster_storage = ClusterStorage::new(client_poll);
    cluster_storage.heartbeat().await;
    sleep(Duration::from_secs(5)).await;
}
