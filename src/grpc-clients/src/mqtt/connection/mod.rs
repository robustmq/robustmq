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

use std::sync::Arc;

use common_base::error::common::CommonError;
use mobc::Connection;

use crate::mqtt::admin::MqttBrokerAdminServiceManager;
use crate::pool::ClientPool;

mod call;
mod inner;

async fn connection_client(
    client_pool: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<MqttBrokerConnectionServiceManager>, CommonError> {
    todo!();
}

#[derive(Clone)]
pub struct MqttBrokerConnectionServiceManager {
    pub addr: String,
}

impl MqttBrokerAdminServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}
