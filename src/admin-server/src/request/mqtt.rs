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

use mqtt_broker::subscribe::{common::Subscriber, manager::ShareLeaderSubscribeData};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct MonitorDataReq {
    pub data_type: String,
    pub topic_name: Option<String>,
    pub client_id: Option<String>,
    pub path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionListReq {
    pub client_id: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicListReq {
    pub topic_name: Option<String>,
    pub topic_type: Option<String>, // "all", "normal", "system"
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicDetailReq {
    pub topic_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribeListReq {
    pub client_id: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SubscribeDetailReq {
    pub client_id: String,
    pub path: String,
}

#[derive(Deserialize, Debug)]
pub struct ShareSubscribeDetailReq {
    pub client_id: String,
    pub group_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribeDetailRep {
    pub share_sub: bool,
    pub group_leader_info: Option<SubGroupLeaderRaw>,
    pub topic_list: Vec<SubTopicRaw>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubTopicRaw {
    pub client_id: String,
    pub path: String,
    pub topic_name: String,
    pub exclusive_push_data: Option<Subscriber>,
    pub share_push_data: Option<ShareLeaderSubscribeData>,
    pub push_thread: Option<SubPushThreadDataRaw>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubGroupLeaderRaw {
    pub broker_id: u64,
    pub broker_addr: String,
    pub extend_info: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubPushThreadDataRaw {
    pub push_success_record_num: u64,
    pub push_error_record_num: u64,
    pub last_push_time: u64,
    pub last_run_time: u64,
    pub create_time: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubPushThreadRaw {}

#[derive(Serialize, Deserialize, Debug)]
pub struct AutoSubscribeListReq {
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CreateAutoSubscribeReq {
    pub topic: String,
    pub qos: u32,
    pub no_local: bool,
    pub retain_as_published: bool,
    pub retained_handling: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DeleteAutoSubscribeReq {
    pub topic_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserListReq {
    pub user_name: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CreateUserReq {
    pub username: String,
    pub password: String,
    pub is_superuser: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DeleteUserReq {
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlackListListReq {
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CreateBlackListReq {
    pub blacklist_type: String,
    pub resource_name: String,
    pub end_time: u64,
    pub desc: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DeleteBlackListReq {
    pub blacklist_type: String,
    pub resource_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicRewriteReq {
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CreateTopicRewriteReq {
    pub action: String,
    pub source_topic: String,
    pub dest_topic: String,
    pub regex: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DeleteTopicRewriteReq {
    pub action: String,
    pub source_topic: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AclListReq {
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CreateAclReq {
    pub resource_type: String,
    pub resource_name: String,
    pub topic: String,
    pub ip: String,
    pub action: String,
    pub permission: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DeleteAclReq {
    pub resource_type: String,
    pub resource_name: String,
    pub topic: String,
    pub ip: String,
    pub action: String,
    pub permission: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectorListReq {
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CreateConnectorReq {
    pub connector_name: String,
    pub connector_type: String,
    pub config: String,
    pub topic_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DeleteConnectorReq {
    pub connector_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SchemaListReq {
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CreateSchemaReq {
    pub schema_name: String,
    pub schema_type: String,
    pub schema: String,
    pub desc: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DeleteSchemaReq {
    pub schema_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SchemaBindListReq {
    pub resource_name: Option<String>,
    pub schema_name: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CreateSchemaBindReq {
    pub schema_name: String,
    pub resource_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DeleteSchemaBindReq {
    pub schema_name: String,
    pub resource_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientListReq {
    pub source_ip: Option<String>,
    pub connection_id: Option<u64>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemAlarmListReq {
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}
