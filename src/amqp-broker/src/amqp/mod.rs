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

use metadata_struct::tenant::DEFAULT_TENANT;

use crate::core::cache::AmqpCacheManager;

pub mod basic;
pub mod channel;
pub mod connection;
pub mod exchange;
pub mod queue;
pub mod route;
pub mod tx;

/// The tenant a connection's operations should run against: its vhost (set at
/// Connection.Open) if known, else DEFAULT_TENANT.
pub(crate) fn tenant_for(amqp_cache: Option<&Arc<AmqpCacheManager>>, connection_id: u64) -> String {
    amqp_cache
        .map(|c| c.tenant_for(connection_id))
        .unwrap_or_else(|| DEFAULT_TENANT.to_string())
}
