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

use crate::manager::SecurityManager;
use common_base::{
    error::ResultCommonError,
    task::{TaskKind, TaskSupervisor},
    tools::loop_select_ticket,
};
use std::sync::Arc;
use tokio::sync::broadcast;

pub fn start_auth_sync_thread(
    security_manager: Arc<SecurityManager>,
    task_supervisor: Arc<TaskSupervisor>,
    stop_send: broadcast::Sender<bool>,
) {
    sync_user_cache(
        security_manager.clone(),
        task_supervisor.clone(),
        stop_send.clone(),
    );
    sync_acl_cache(
        security_manager.clone(),
        task_supervisor.clone(),
        stop_send.clone(),
    );
    sync_blacklist_cache(security_manager, task_supervisor.clone(), stop_send);
}

fn sync_user_cache(
    security_manager: Arc<SecurityManager>,
    task_supervisor: Arc<TaskSupervisor>,
    stop_send: broadcast::Sender<bool>,
) {
    task_supervisor.spawn(TaskKind::MQTTSecurityUserSync.to_string(), async move {
        let ac_fn = async || -> ResultCommonError {
            for driver in security_manager.drivers_list().await? {
                let list = driver.read_all_user().await?;
                for user in list.iter() {
                    security_manager.security_metadata.add_user(user.clone());
                }
            }
            Ok(())
        };
        loop_select_ticket(ac_fn, 5000, &stop_send).await;
    });
}

fn sync_acl_cache(
    security_manager: Arc<SecurityManager>,
    task_supervisor: Arc<TaskSupervisor>,
    stop_send: broadcast::Sender<bool>,
) {
    task_supervisor.spawn(TaskKind::MQTTSecurityAclSync.to_string(), async move {
        let ac_fn = async || -> ResultCommonError {
            for driver in security_manager.drivers_list().await? {
                let list = driver.read_all_acl().await?;
                for acl in list.iter() {
                    security_manager.security_metadata.add_acl(acl.to_owned());
                }
            }
            Ok(())
        };
        loop_select_ticket(ac_fn, 5000, &stop_send).await;
    });
}

fn sync_blacklist_cache(
    security_manager: Arc<SecurityManager>,
    task_supervisor: Arc<TaskSupervisor>,
    stop_send: broadcast::Sender<bool>,
) {
    task_supervisor.spawn(
        TaskKind::MQTTSecurityBlacklistSync.to_string(),
        async move {
            let ac_fn = async || -> ResultCommonError {
                for driver in security_manager.drivers_list().await? {
                    let list = driver.read_all_blacklist().await?;
                    for blacklist in list.iter() {
                        security_manager
                            .security_metadata
                            .add_blacklist(blacklist.to_owned());
                    }
                }
                Ok(())
            };
            loop_select_ticket(ac_fn, 5000, &stop_send).await;
        },
    );
}
