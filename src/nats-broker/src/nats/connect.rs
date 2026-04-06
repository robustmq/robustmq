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

use common_config::broker::broker_config;
use protocol::nats::packet::{ClientConnect, NatsPacket};

use crate::core::connection::NatsConnection;
use crate::core::error::NatsProtocolError;
use crate::core::security::login_check;
use crate::core::tenant::get_tenant;
use crate::handler::command::NatsProcessContext;

pub fn process_connect(ctx: &NatsProcessContext, req: &ClientConnect) -> Result<(), NatsPacket> {
    let auth_required = broker_config().nats_runtime.auth_required;

    let authed = login_check(
        &ctx.security_manager,
        &get_tenant(),
        auth_required,
        req.user.as_deref(),
        req.pass.as_deref(),
    );

    if !authed {
        return Err(NatsPacket::Err(
            NatsProtocolError::AuthorizationViolation.message(),
        ));
    }

    let mut connection = NatsConnection::new(ctx.connect_id, String::new());
    connection.verbose = req.verbose;
    connection.pedantic = req.pedantic;
    connection.echo = req.echo.unwrap_or(true);
    connection.headers = req.headers.unwrap_or(false);
    connection.no_responders = req.no_responders.unwrap_or(false);
    connection.protocol = req.protocol;
    connection.client_name = req.name.clone();
    connection.lang = req.lang.clone();
    connection.version = req.version.clone();

    if auth_required {
        connection.login_success(req.user.clone().unwrap_or_default());
    }

    ctx.cache_manager.add_connection(connection);
    Ok(())
}
