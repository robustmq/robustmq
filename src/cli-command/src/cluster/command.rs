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
use admin_server::{
    client::AdminHttpClient,
    cluster::{tenant::TenantListRow, ClusterConfigSetReq, ClusterInfoResp},
};
use chrono::{Local, TimeZone};
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
    ListTenant,
    CreateTenant {
        tenant_name: String,
        desc: Option<String>,
    },
    DeleteTenant {
        tenant_name: String,
    },
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
            ClusterActionType::ListTenant => {
                self.list_tenant(params).await;
            }
            ClusterActionType::CreateTenant { tenant_name, desc } => {
                self.create_tenant(params, tenant_name, desc).await;
            }
            ClusterActionType::DeleteTenant { tenant_name } => {
                self.delete_tenant(params, tenant_name).await;
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
                if matches!(params.output, OutputFormat::Json) {
                    match serde_json::from_str::<serde_json::Value>(&raw) {
                        Ok(v) => println!("{}", serde_json::to_string_pretty(&v).unwrap_or(raw)),
                        Err(_) => println!("{raw}"),
                    }
                    return;
                }
                // Parse into ClusterInfoResp for table output
                let info: ClusterInfoResp = match serde_json::from_str::<serde_json::Value>(&raw)
                    .ok()
                    .and_then(|v| v.get("data").cloned())
                    .and_then(|d| serde_json::from_value(d).ok())
                {
                    Some(v) => v,
                    None => {
                        println!("{raw}");
                        return;
                    }
                };

                // ── Cluster Info ──────────────────────────────────────────
                println!("Cluster: {}", info.cluster_name);
                println!("Version: {}", info.version);
                println!("Started: {}", format_timestamp(info.start_time),);
                let mut nodes: Vec<String> = info.nodes.into_iter().collect();
                nodes.sort();
                println!("Nodes:   {}", nodes.join(", "));
                println!();

                // ── Broker Nodes ──────────────────────────────────────────
                println!("=== Broker Nodes ===");
                let mut node_table = Table::new();
                node_table.set_titles(row![
                    "node_id",
                    "ip",
                    "roles",
                    "grpc_addr",
                    "mqtt_addr",
                    "start_time"
                ]);
                for node in &info.broker_node_list {
                    let roles = node.roles.join(",");
                    let mqtt_addr = node.extend.mqtt.mqtt_addr.as_str();
                    node_table.add_row(row![
                        node.node_id,
                        node.node_ip,
                        roles,
                        node.grpc_addr,
                        mqtt_addr,
                        format_timestamp(node.start_time),
                    ]);
                }
                node_table.printstd();
                println!();

                // ── Raft Shards ───────────────────────────────────────────
                println!("=== Raft Shards ===");
                let mut raft_table = Table::new();
                raft_table.set_titles(row![
                    "shard",
                    "state",
                    "leader",
                    "term",
                    "log_index",
                    "applied"
                ]);
                let mut shards: Vec<(&String, _)> = info.meta.iter().collect();
                shards.sort_by_key(|(k, _)| *k);
                for (shard, status) in shards {
                    raft_table.add_row(row![
                        shard,
                        status.state,
                        status.current_leader,
                        status.current_term,
                        status.last_log_index,
                        status.last_applied.index,
                    ]);
                }
                raft_table.printstd();
            }
            Err(e) => error_info(e.to_string()),
        }
    }

    // ------------ tenant ------------
    async fn list_tenant(&self, params: ClusterCliCommandParam) {
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));
        let request = admin_server::cluster::tenant::TenantListReq {
            tenant_name: None,
            limit: None,
            page: None,
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };
        match admin_client
            .get_tenant_list::<_, Vec<TenantListRow>>(&request)
            .await
        {
            Ok(page_data) => {
                if matches!(params.output, OutputFormat::Json) {
                    self.print_json(&page_data);
                    return;
                }
                let mut table = Table::new();
                table.set_titles(row!["tenant_name", "desc", "create_time"]);
                for tenant in page_data.data {
                    table.add_row(row![tenant.tenant_name, tenant.desc, tenant.create_time]);
                }
                table.printstd();
            }
            Err(e) => {
                println!("List tenant exception");
                error_info(e.to_string());
            }
        }
    }

    async fn create_tenant(
        &self,
        params: ClusterCliCommandParam,
        tenant_name: String,
        desc: Option<String>,
    ) {
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));
        let request = admin_server::cluster::tenant::CreateTenantReq {
            tenant_name,
            desc,
            config: None,
        };
        match admin_client.create_tenant(&request).await {
            Ok(_) => println!("Created successfully!"),
            Err(e) => {
                println!("Create tenant exception");
                error_info(e.to_string());
            }
        }
    }

    async fn delete_tenant(&self, params: ClusterCliCommandParam, tenant_name: String) {
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));
        let request = admin_server::cluster::tenant::DeleteTenantReq { tenant_name };
        match admin_client.delete_tenant(&request).await {
            Ok(_) => println!("Deleted successfully!"),
            Err(e) => {
                println!("Delete tenant exception");
                error_info(e.to_string());
            }
        }
    }
}

fn format_timestamp(secs: u64) -> String {
    match Local.timestamp_opt(secs as i64, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        _ => secs.to_string(),
    }
}
