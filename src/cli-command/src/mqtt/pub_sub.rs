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

use paho_mqtt::{
    Client, ConnectOptions, ConnectOptionsBuilder, CreateOptions, CreateOptionsBuilder, Properties,
    PropertyCode, SslOptionsBuilder,
};
use std::process;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
pub struct PublishArgsRequest {
    pub topic: String,
    pub qos: i32,
    pub retained: bool,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubscribeArgsRequest {
    pub topic: String,
    pub qos: i32,
    pub username: String,
    pub password: String,
}

pub(crate) fn error_info(err: String) {
    println!("Exception:{err}");
}

pub fn connect_server5(
    client_id: &str,
    username: String,
    password: String,
    addr: &str,
    ws: bool,
    ssl: bool,
) -> Client {
    let props = build_v5_pros();
    let create_opts = build_create_pros(client_id, addr);
    let cli = Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {err:?}");
        process::exit(1);
    });
    let conn_opts = build_v5_conn_pros(props, username, password, ws, ssl);
    match cli.connect(conn_opts) {
        Ok(response) => {
            let resp = response.connect_response().unwrap();
            println!("able to connect: {:?}", resp.server_uri);
        }
        Err(e) => {
            println!("Unable to connect: {e:?}");
            process::exit(1);
        }
    }
    cli
}

pub fn build_v5_pros() -> Properties {
    let mut props = Properties::new();
    props
        .push_u32(PropertyCode::SessionExpiryInterval, 3)
        .unwrap();
    props.push_u16(PropertyCode::ReceiveMaximum, 128).unwrap();
    props
        .push_u32(PropertyCode::MaximumPacketSize, 2048)
        .unwrap();
    props
        .push_u16(PropertyCode::TopicAliasMaximum, 128)
        .unwrap();
    props
        .push_val(PropertyCode::RequestResponseInformation, 0)
        .unwrap();
    props
        .push_val(PropertyCode::RequestProblemInformation, 1)
        .unwrap();
    props
}

pub fn build_create_pros(client_id: &str, addr: &str) -> CreateOptions {
    if client_id.is_empty() {
        CreateOptionsBuilder::new().server_uri(addr).finalize()
    } else {
        CreateOptionsBuilder::new()
            .server_uri(addr)
            .client_id(client_id)
            .finalize()
    }
}

pub fn build_v5_conn_pros(
    props: Properties,
    username: String,
    password: String,
    ws: bool,
    ssl: bool,
) -> ConnectOptions {
    let mut conn_opts = if ws {
        ConnectOptionsBuilder::new_ws_v5()
    } else {
        ConnectOptionsBuilder::new_v5()
    };
    if ssl {
        let ssl_opts = SslOptionsBuilder::new()
            .trust_store(format!(
                "{}/../config/example/certs/ca.pem",
                env!("CARGO_MANIFEST_DIR")
            ))
            .unwrap()
            .verify(false)
            .disable_default_trust_store(false)
            .finalize();
        conn_opts.ssl_options(ssl_opts);
    }
    conn_opts
        .keep_alive_interval(Duration::from_secs(600))
        .clean_start(true)
        .connect_timeout(Duration::from_secs(60))
        .properties(props)
        .user_name(username)
        .password(password)
        .finalize()
}
