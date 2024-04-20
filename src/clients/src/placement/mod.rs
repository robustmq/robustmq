use self::{journal::journal_interface_call, kv::kv_interface_call, placement::placement_interface_call};
use crate::{retry_sleep_time, retry_times, ClientPool};
use common_base::errors::RobustMQError;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};

#[derive(Clone)]
pub enum PlacementCenterService {
    Journal,
    Kv,
    Placement,
}

#[derive(Clone)]
pub enum PlacementCenterInterface {
    // kv interface
    Set,
    Get,
    Delete,
    Exists,

    // placement inner interface
    RegisterNode,
    UnRegisterNode,
    Heartbeat,
    SendRaftMessage,
    SendRaftConfChange,

    // journal service interface
    CreateShard,
    DeleteShard,
    CreateSegment,
    DeleteSegment,
}

pub mod journal;
pub mod kv;
pub mod placement;

async fn retry_call(
    service: PlacementCenterService,
    interface: PlacementCenterInterface,
    client_poll: Arc<Mutex<ClientPool>>,
    addrs: Vec<String>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    let mut times = 0;
    loop {
        let index = times % addrs.len();
        let addr = addrs.get(index).unwrap().clone();
        let result = match service {
            PlacementCenterService::Journal => {
                journal_interface_call(
                    interface.clone(),
                    client_poll.clone(),
                    addr,
                    request.clone(),
                )
                .await
            }

            PlacementCenterService::Kv => {
                kv_interface_call(
                    interface.clone(),
                    client_poll.clone(),
                    addr,
                    request.clone(),
                )
                .await
            }

            PlacementCenterService::Placement => {
                placement_interface_call(
                    interface.clone(),
                    client_poll.clone(),
                    addr,
                    request.clone(),
                )
                .await
            }
        };

        match result {
            Ok(data) => {
                return Ok(data);
            }
            Err(e) => {
                if times > retry_times() {
                    return Err(e);
                }
                times = times + 1;
            }
        }
        sleep(Duration::from_secs(retry_sleep_time(times) as u64)).await;
    }
}
