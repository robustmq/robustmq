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

use self::{
    journal::journal_interface_call, kv::kv_interface_call, mqtt::mqtt_interface_call,
    placement::placement_interface_call,
};
use crate::{poll::ClientPool, retry_sleep_time, retry_times};
use common_base::error::common::CommonError;
use lazy_static::lazy_static;
use log::error;
use std::{collections::HashSet, sync::Arc, time::Duration};
use tokio::time::sleep;

#[derive(Clone, Debug)]
pub enum PlacementCenterService {
    Journal,
    Kv,
    Placement,
    Mqtt,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PlacementCenterInterface {
    // kv interface
    Set,
    Get,
    Delete,
    Exists,

    // placement inner interface
    ClusterStatus,
    ListNode,
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

    // mqtt service interface
    GetShareSubLeader,
    CreateUser,
    DeleteUser,
    ListUser,
    CreateTopic,
    DeleteTopic,
    ListTopic,
    SetTopicRetainMessage,
    CreateSession,
    DeleteSession,
    ListSession,
    UpdateSession,
    SaveLastWillMessage,
    SetReourceConfig,
    GetReourceConfig,
    DeleteReourceConfig,
    SetIdempotentData,
    ExistsIdempotentData,
    DeleteIdempotentData,
    CreateAcl,
    DeleteAcl,
    ListAcl,
    CreateBlackList,
    DeleteBlackList,
    ListBlackList,
}

impl PlacementCenterInterface {
    pub fn should_forward_to_leader(&self) -> bool {
        lazy_static! {
            static ref FORWARD_SET: HashSet<PlacementCenterInterface> = {
                let mut set = HashSet::new();
                set.insert(PlacementCenterInterface::CreateUser);
                set.insert(PlacementCenterInterface::DeleteUser);
                set.insert(PlacementCenterInterface::CreateTopic);
                set.insert(PlacementCenterInterface::DeleteTopic);
                set.insert(PlacementCenterInterface::CreateSession);
                set.insert(PlacementCenterInterface::DeleteSession);
                set.insert(PlacementCenterInterface::UpdateSession);
                set.insert(PlacementCenterInterface::DeleteSession);
                set.insert(PlacementCenterInterface::CreateAcl);
                set.insert(PlacementCenterInterface::DeleteAcl);
                set.insert(PlacementCenterInterface::CreateBlackList);
                set.insert(PlacementCenterInterface::DeleteBlackList);
                set
            };
        }
        FORWARD_SET.contains(self)
    }

    pub fn get_inner_function_name(&self) -> String {
        let enum_name = format!("{:?}", self);
        let mut result = String::from("inner_");
        for (i, c) in enum_name.chars().enumerate() {
            if i > 0 && c.is_uppercase() {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        }
        result
    }
}

pub mod journal;
pub mod kv;
pub mod mqtt;
pub mod placement;

async fn retry_call(
    service: PlacementCenterService,
    interface: PlacementCenterInterface,
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    let mut times = 1;
    loop {
        let index = times % addrs.len();
        let addr = addrs.get(index).unwrap().clone();
        let result = match service {
            PlacementCenterService::Journal => {
                journal_interface_call(
                    interface.clone(),
                    client_poll.clone(),
                    addr.clone(),
                    request.clone(),
                )
                .await
            }

            PlacementCenterService::Kv => {
                kv_interface_call(
                    interface.clone(),
                    client_poll.clone(),
                    addr.clone(),
                    request.clone(),
                )
                .await
            }

            PlacementCenterService::Placement => {
                placement_interface_call(
                    interface.clone(),
                    client_poll.clone(),
                    addr.clone(),
                    request.clone(),
                )
                .await
            }

            PlacementCenterService::Mqtt => {
                mqtt_interface_call(
                    interface.clone(),
                    client_poll.clone(),
                    addr.clone(),
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
                error!(
                    "{:?}@{:?}@{},{},",
                    service.clone(),
                    interface.clone(),
                    addr.clone(),
                    e
                );
                if times > retry_times() {
                    return Err(e);
                }
                times = times + 1;
            }
        }
        sleep(Duration::from_secs(retry_sleep_time(times) as u64)).await;
    }
}
