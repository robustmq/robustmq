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

use serde::Deserialize;

#[derive(Deserialize)]
pub struct OverviewMetricsReq {
    pub start_time: u64,
    pub end_time: u64,
}

#[derive(Deserialize, Debug)]
pub struct SessionListReq {
    pub client_id: Option<String>,
    pub page_num: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TopicListReq {
    pub topic_name: Option<String>,
    pub page_num: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SubscribeListReq {
    pub client_id: Option<String>,
    pub page_num: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}
#[derive(Deserialize, Debug)]
pub struct SubscribeDetailReq {}

#[derive(Deserialize, Debug)]
pub struct AutoSubscribeListReq {
    pub page_num: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CreateAutoSubscribeReq {
    pub topic: String,
    pub qos: u32,
    pub no_local: bool,
    pub retain_as_published: bool,
    pub retained_handling: u32,
}

#[derive(Deserialize, Debug)]
pub struct DeleteAutoSubscribeReq {
    pub topic_name: String,
}

#[derive(Deserialize, Debug)]
pub struct UserListReq {
    pub user_name: Option<String>,
    pub page_num: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CreateUserReq {
    pub username: String,
    pub password: String,
    pub is_superuser: bool,
}

#[derive(Deserialize, Debug)]
pub struct DeleteUserReq {
    pub username: String,
}

#[derive(Deserialize, Debug)]
pub struct BlackListListReq {
    pub page_num: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CreateBlackListReq {
    pub blacklist_type: String,
    pub resource_name: String,
    pub end_time: u64,
    pub desc: String,
}

#[derive(Deserialize, Debug)]
pub struct DeleteBlackListReq {
    pub blacklist_type: String,
    pub resource_name: String,
}

#[derive(Deserialize, Debug)]
pub struct TopicRewriteReq {
    pub page_num: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CreateTopicRewriteReq {
    pub action: String,
    pub source_topic: String,
    pub dest_topic: String,
    pub regex: String,
}

#[derive(Deserialize, Debug)]
pub struct DeleteTopicRewriteReq {
    pub action: String,
    pub source_topic: String,
}

#[derive(Deserialize, Debug)]
pub struct AclListReq {
    pub page_num: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CreateAclReq {
    pub resource_type: String,
    pub resource_name: String,
    pub topic: String,
    pub ip: String,
    pub action: String,
    pub permission: String,
}

#[derive(Deserialize, Debug)]
pub struct DeleteAclReq {
    pub resource_type: String,
    pub resource_name: String,
    pub topic: String,
    pub ip: String,
    pub action: String,
    pub permission: String,
}

#[derive(Deserialize, Debug)]
pub struct ConnectorListReq {
    pub page_num: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CreateConnectorReq {
    pub connector_name: String,
    pub connector_type: String,
    pub config: String,
    pub topic_id: String,
}

#[derive(Deserialize, Debug)]
pub struct DeleteConnectorReq {
    pub connector_name: String,
}

#[derive(Deserialize, Debug)]
pub struct SchemaListReq {
    pub page_num: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CreateSchemaReq {
    pub schema_name: String,
    pub schema_type: String,
    pub schema: String,
    pub desc: String,
}

#[derive(Deserialize, Debug)]
pub struct DeleteSchemaReq {
    pub schema_name: String,
}

#[derive(Deserialize, Debug)]
pub struct SchemaBindListReq {
    pub resource_name: Option<String>,
    pub schema_name: Option<String>,
    pub page_num: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CreateSchemaBindReq {
    pub schema_name: String,
    pub resource_name: String,
}

#[derive(Deserialize, Debug)]
pub struct DeleteSchemaBindReq {
    pub schema_name: String,
    pub resource_name: String,
}

#[derive(Deserialize, Debug)]
pub struct ClientListReq {
    pub source_ip: Option<String>,
    pub connection_id: Option<u64>,
    pub page_num: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SystemAlarmListReq {
    pub page_num: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ClusterConfigSetReq {
    pub config_type: String,
    pub config: String,
}
