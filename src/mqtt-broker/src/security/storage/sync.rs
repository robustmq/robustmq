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

use crate::security::AuthDriver;
use common_base::{
    error::{common::CommonError, ResultCommonError},
    tools::loop_select_ticket,
};
use std::sync::Arc;
use tokio::sync::broadcast;

pub fn sync_auth_storage_info(auth_driver: Arc<AuthDriver>, stop_send: broadcast::Sender<bool>) {
    sync_user_cache(auth_driver.clone(), stop_send.clone());
    sync_acl_cache(auth_driver.clone(), stop_send.clone());
    sync_blacklist_cache(auth_driver, stop_send);
}

fn sync_user_cache(auth_driver: Arc<AuthDriver>, stop_send: broadcast::Sender<bool>) {
    tokio::spawn(async move {
        let ac_fn = async || -> ResultCommonError {
            if let Err(e) = auth_driver.update_user_cache().await {
                return Err(CommonError::CommonError(e.to_string()));
            }
            Ok(())
        };
        loop_select_ticket(ac_fn, 1000, &stop_send).await;
    });
}

fn sync_acl_cache(auth_driver: Arc<AuthDriver>, stop_send: broadcast::Sender<bool>) {
    tokio::spawn(async move {
        let ac_fn = async || -> ResultCommonError {
            if let Err(e) = auth_driver.update_acl_cache().await {
                return Err(CommonError::CommonError(e.to_string()));
            }
            Ok(())
        };
        loop_select_ticket(ac_fn, 1000, &stop_send).await;
    });
}

fn sync_blacklist_cache(auth_driver: Arc<AuthDriver>, stop_send: broadcast::Sender<bool>) {
    tokio::spawn(async move {
        let ac_fn = async || -> ResultCommonError {
            if let Err(e) = auth_driver.update_blacklist_cache().await {
                return Err(CommonError::CommonError(e.to_string()));
            }
            Ok(())
        };
        loop_select_ticket(ac_fn, 1000, &stop_send).await;
    });
}
