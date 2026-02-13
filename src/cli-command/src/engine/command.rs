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
use admin_server::client::AdminHttpClient;
use admin_server::engine::shard::{
    CommitOffsetReq, GetOffsetByGroupReq, GetOffsetByGroupResp, GetOffsetByTimestampReq,
    GetOffsetByTimestampResp, SegmentListReq, SegmentListResp, ShardCreateReq, ShardDeleteReq,
    ShardListReq, ShardListRow,
};
use prettytable::{row, Table};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone)]
pub struct EngineCliCommandParam {
    pub server: String,
    pub output: OutputFormat,
    pub page: u32,
    pub limit: u32,
    pub action: EngineActionType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EngineActionType {
    ShardList {
        shard_name: Option<String>,
    },
    ShardCreate {
        shard_name: String,
        config: String,
    },
    ShardDelete {
        shard_name: String,
    },
    SegmentList {
        shard_name: String,
    },
    OffsetByTimestamp {
        shard_name: String,
        timestamp: u64,
        strategy: String,
    },
    OffsetByGroup {
        group_name: String,
    },
    CommitOffset {
        group_name: String,
        offsets: HashMap<String, u64>,
    },
}

pub struct EngineCommand;

impl Default for EngineCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineCommand {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(&self, params: EngineCliCommandParam) {
        match params.action.clone() {
            EngineActionType::ShardList { shard_name } => self.shard_list(params, shard_name).await,
            EngineActionType::ShardCreate { shard_name, config } => {
                self.shard_create(params, shard_name, config).await
            }
            EngineActionType::ShardDelete { shard_name } => {
                self.shard_delete(params, shard_name).await
            }
            EngineActionType::SegmentList { shard_name } => {
                self.segment_list(params, shard_name).await
            }
            EngineActionType::OffsetByTimestamp {
                shard_name,
                timestamp,
                strategy,
            } => {
                self.offset_by_timestamp(params, shard_name, timestamp, strategy)
                    .await
            }
            EngineActionType::OffsetByGroup { group_name } => {
                self.offset_by_group(params, group_name).await
            }
            EngineActionType::CommitOffset {
                group_name,
                offsets,
            } => self.commit_offset(params, group_name, offsets).await,
        }
    }

    fn print_json<T: Serialize>(&self, value: &T) {
        match serde_json::to_string_pretty(value) {
            Ok(raw) => println!("{raw}"),
            Err(e) => error_info(e.to_string()),
        }
    }

    fn is_json(&self, output: &OutputFormat) -> bool {
        matches!(output, OutputFormat::Json)
    }

    async fn shard_list(&self, params: EngineCliCommandParam, shard_name: Option<String>) {
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));
        let request = ShardListReq {
            shard_name,
            limit: Some(params.limit),
            page: Some(params.page),
            sort_field: None,
            sort_by: None,
            filter_field: None,
            filter_values: None,
            exact_match: None,
        };

        match admin_client
            .get_shard_list::<ShardListReq, Vec<ShardListRow>>(&request)
            .await
        {
            Ok(page_data) => {
                if self.is_json(&params.output) {
                    self.print_json(&page_data);
                    return;
                }
                let mut table = Table::new();
                table.set_titles(row![
                    "shard_name",
                    "status",
                    "replica_num",
                    "storage_type",
                    "retention_sec"
                ]);
                for item in page_data.data {
                    table.add_row(row![
                        item.shard_info.shard_name,
                        format!("{:?}", item.shard_info.status),
                        item.shard_info.config.replica_num,
                        format!("{:?}", item.shard_info.config.storage_type),
                        item.shard_info.config.retention_sec
                    ]);
                }
                table.printstd();
            }
            Err(e) => error_info(e.to_string()),
        }
    }

    async fn shard_create(
        &self,
        params: EngineCliCommandParam,
        shard_name: String,
        config: String,
    ) {
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));
        let request = ShardCreateReq { shard_name, config };
        match admin_client.create_shard(&request).await {
            Ok(raw) => println!("{raw}"),
            Err(e) => error_info(e.to_string()),
        }
    }

    async fn shard_delete(&self, params: EngineCliCommandParam, shard_name: String) {
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));
        let request = ShardDeleteReq { shard_name };
        match admin_client.delete_shard(&request).await {
            Ok(raw) => println!("{raw}"),
            Err(e) => error_info(e.to_string()),
        }
    }

    async fn segment_list(&self, params: EngineCliCommandParam, shard_name: String) {
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));
        let request = SegmentListReq { shard_name };
        match admin_client.get_segment_list(&request).await {
            Ok(raw) => {
                if self.is_json(&params.output) {
                    println!("{raw}");
                    return;
                }
                match serde_json::from_str::<SegmentListResp>(&raw) {
                    Ok(resp) => {
                        let mut table = Table::new();
                        table.set_titles(row![
                            "segment_seq",
                            "shard_name",
                            "leader",
                            "leader_epoch",
                            "status"
                        ]);
                        for item in resp.segment_list {
                            table.add_row(row![
                                item.segment.segment_seq,
                                item.segment.shard_name,
                                item.segment.leader,
                                item.segment.leader_epoch,
                                format!("{:?}", item.segment.status)
                            ]);
                        }
                        table.printstd();
                    }
                    Err(_) => println!("{raw}"),
                }
            }
            Err(e) => error_info(e.to_string()),
        }
    }

    async fn offset_by_timestamp(
        &self,
        params: EngineCliCommandParam,
        shard_name: String,
        timestamp: u64,
        strategy: String,
    ) {
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));
        let request = GetOffsetByTimestampReq {
            shard_name,
            timestamp,
            strategy,
        };
        match admin_client
            .get_offset_by_timestamp::<GetOffsetByTimestampReq, GetOffsetByTimestampResp>(&request)
            .await
        {
            Ok(resp) => {
                if self.is_json(&params.output) {
                    self.print_json(&resp);
                } else {
                    let mut table = Table::new();
                    table.set_titles(row!["offset"]);
                    table.add_row(row![resp.offset]);
                    table.printstd();
                }
            }
            Err(e) => error_info(e.to_string()),
        }
    }

    async fn offset_by_group(&self, params: EngineCliCommandParam, group_name: String) {
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));
        let request = GetOffsetByGroupReq { group_name };
        match admin_client
            .get_offset_by_group::<GetOffsetByGroupReq, GetOffsetByGroupResp>(&request)
            .await
        {
            Ok(resp) => {
                if self.is_json(&params.output) {
                    self.print_json(&resp);
                    return;
                }
                let mut table = Table::new();
                table.set_titles(row!["shard_name", "offset"]);
                for item in resp.offsets {
                    table.add_row(row![item.shard_name, item.offset]);
                }
                table.printstd();
            }
            Err(e) => error_info(e.to_string()),
        }
    }

    async fn commit_offset(
        &self,
        params: EngineCliCommandParam,
        group_name: String,
        offsets: HashMap<String, u64>,
    ) {
        let admin_client = AdminHttpClient::new(format!("http://{}", params.server));
        let request = CommitOffsetReq {
            group_name,
            offsets,
        };
        match admin_client.commit_offset(&request).await {
            Ok(raw) => println!("{raw}"),
            Err(e) => error_info(e.to_string()),
        }
    }
}
