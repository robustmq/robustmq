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

use crate::mqtt::pub_sub::error_info;
use crate::output::OutputFormat;
use admin_server::{client::AdminHttpClient, cluster::ClusterConfigSetReq};
use common_config::config::BrokerConfig;
use prettytable::{row, Table};
use serde::Serialize;

#[derive(Clone)]
pub struct ClusterCliCommandParam {
    pub server: String,
    pub output: OutputFormat,
    pub action: ClusterActionType,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ClusterActionType {
    Status,
    Healthy,
    GetConfig,
    SetConfig(ClusterConfigSetReq),
}

pub struct ClusterCommand {}

impl Default for ClusterCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl ClusterCommand {
    pub fn new() -> Self {
        ClusterCommand {}
    }
    pub async fn start(&self, params: ClusterCliCommandParam) {
        match params.action.clone() {
            ClusterActionType::Status => {
                self.status(params).await;
            }
            ClusterActionType::Healthy => {
                self.healthy(params).await;
            }
            ClusterActionType::GetConfig => {
                self.get_cluster_config(params).await;
            }
            ClusterActionType::SetConfig(request) => {
                self.set_cluster_config(params, request.clone()).await;
            }
        }
    }

    fn print_json<T: Serialize>(&self, data: &T) {
        match serde_json::to_string_pretty(data) {
            Ok(raw) => println!("{raw}"),
            Err(e) => error_info(e.to_string()),
        }
    }

    async fn set_cluster_config(
        &self,
        params: ClusterCliCommandParam,
        cli_request: admin_server::cluster::ClusterConfigSetReq,
    ) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.set_cluster_config(&cli_request).await {
            Ok(_) => {
                println!("Cluster configuration set successfully!");
            }
            Err(e) => {
                println!("MQTT broker set cluster config exception");
                error_info(e.to_string());
            }
        }
    }

    async fn get_cluster_config(&self, params: ClusterCliCommandParam) {
        // Create admin HTTP client
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));

        match admin_client.get_cluster_config().await {
            Ok(response_text) => {
                // Try to parse the response as BrokerConfig
                match serde_json::from_str::<BrokerConfig>(&response_text) {
                    Ok(data) => {
                        let json = match serde_json::to_string_pretty(&data) {
                            Ok(data) => data,
                            Err(e) => {
                                println!("MQTT broker cluster normal exception");
                                error_info(e.to_string());
                                return;
                            }
                        };
                        println!("{json}");
                    }
                    Err(_) => {
                        // If direct parsing fails, try to parse as the original response format
                        println!("{response_text}");
                    }
                }
            }
            Err(e) => {
                println!("MQTT broker cluster normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn healthy(&self, params: ClusterCliCommandParam) {
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));
        match admin_client.get_cluster_healthy().await {
            Ok(healthy) => {
                if matches!(params.output, OutputFormat::Json) {
                    self.print_json(&healthy);
                } else {
                    let mut table = Table::new();
                    table.set_titles(row!["healthy"]);
                    table.add_row(row![healthy]);
                    table.printstd();
                }
            }
            Err(e) => error_info(e.to_string()),
        }
    }

    async fn status(&self, params: ClusterCliCommandParam) {
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));
        match admin_client.get_status().await {
            Ok(raw) => {
                let _ = params.output;
                println!("{raw}");
            }
            Err(e) => error_info(e.to_string()),
        }
    }
}
