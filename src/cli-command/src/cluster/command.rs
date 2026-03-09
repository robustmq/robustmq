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
    cluster::{tenant::TenantListRow, ClusterConfigSetReq},
};
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
                let _ = params.output;
                println!("{raw}");
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
        let request = admin_server::cluster::tenant::CreateTenantReq { tenant_name, desc };
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
