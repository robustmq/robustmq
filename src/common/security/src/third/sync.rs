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

use common_base::{
    error::{common::CommonError, ResultCommonError},
    task::{TaskKind, TaskSupervisor},
    tools::loop_select_ticket,
};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::manager::SecurityManager;

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
            if let Err(e) = security_manager.update_user_cache().await {
                return Err(CommonError::CommonError(e.to_string()));
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
            if let Err(e) = security_manager.update_acl_cache().await {
                return Err(CommonError::CommonError(e.to_string()));
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
                if let Err(e) = security_manager.update_blacklist_cache().await {
                    return Err(CommonError::CommonError(e.to_string()));
                }
                Ok(())
            };
            loop_select_ticket(ac_fn, 5000, &stop_send).await;
        },
    );
}
