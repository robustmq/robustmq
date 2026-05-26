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

use crate::mq9::params::{
    CreateForwardRuleArgs, DeleteForwardRuleArgs, DetailForwardRuleArgs, ForwardRuleAction,
    ListForwardRuleArgs, ToggleForwardRuleArgs, UpdateForwardRuleArgs,
};
use admin_server::client::AdminHttpClient;
use admin_server::mq9::forward_rule::{
    CreateForwardRuleReq, DeleteForwardRuleReq, ForwardRuleDetailReq, ForwardRuleListReq,
    ForwardRuleListRow, UpdateForwardRuleReq,
};
use metadata_struct::connector::rule::ETLRule;
use prettytable::{row, Table};

pub struct Mq9Command {
    server: String,
    page: u32,
    limit: u32,
}

impl Mq9Command {
    pub fn new(server: String, page: u32, limit: u32) -> Self {
        Self {
            server,
            page,
            limit,
        }
    }

    pub async fn forward_rule(&self, action: ForwardRuleAction) {
        match action {
            ForwardRuleAction::List(args) => self.list_forward_rule(args).await,
            ForwardRuleAction::Detail(args) => self.detail_forward_rule(args).await,
            ForwardRuleAction::Create(args) => self.create_forward_rule(args).await,
            ForwardRuleAction::Update(args) => self.update_forward_rule(args).await,
            ForwardRuleAction::Delete(args) => self.delete_forward_rule(args).await,
            ForwardRuleAction::Enable(args) => self.toggle_forward_rule(args, true).await,
            ForwardRuleAction::Disable(args) => self.toggle_forward_rule(args, false).await,
        }
    }

    fn client(&self) -> AdminHttpClient {
        AdminHttpClient::new(format!("http://{}", self.server))
    }

    async fn list_forward_rule(&self, args: ListForwardRuleArgs) {
        let req = ForwardRuleListReq {
            tenant: args.tenant,
            rule_name: args.rule_name,
            topic_name: args.topic_name,
            enabled: args.enabled,
            limit: Some(self.limit),
            page: Some(self.page),
            sort_field: None,
            sort_by: None,
        };
        match self
            .client()
            .get_mq9_forward_rule_list::<_, ForwardRuleListRow>(&req)
            .await
        {
            Ok(page_data) => {
                println!("mq9 forward rule list:");
                let mut table = Table::new();
                table.set_titles(row![
                    "tenant",
                    "rule_name",
                    "topic_name",
                    "mail_prefixes",
                    "tags",
                    "priorities",
                    "sender_prefixes",
                    "keep_headers",
                    "on_failure",
                    "enabled",
                    "create_time",
                    "update_time",
                ]);
                for row in page_data.data {
                    table.add_row(row![
                        row.tenant,
                        row.rule_name,
                        row.topic_name,
                        row.mail_address_prefixes.join(","),
                        row.any_tags.join(","),
                        row.priorities.join(","),
                        row.sender_prefixes.join(","),
                        row.keep_headers,
                        row.on_failure,
                        row.enabled,
                        row.create_time,
                        row.update_time,
                    ]);
                }
                table.printstd();
            }
            Err(e) => eprintln!("list forward rule failed: {e}"),
        }
    }

    async fn detail_forward_rule(&self, args: DetailForwardRuleArgs) {
        let req = ForwardRuleDetailReq {
            tenant: args.tenant,
            rule_name: args.rule_name,
        };
        match self
            .client()
            .get_mq9_forward_rule_detail::<_, ForwardRuleListRow>(&req)
            .await
        {
            Ok(row) => match serde_json::to_string_pretty(&row) {
                Ok(s) => println!("{s}"),
                Err(e) => eprintln!("serialize forward rule failed: {e}"),
            },
            Err(e) => eprintln!("get forward rule failed: {e}"),
        }
    }

    async fn create_forward_rule(&self, args: CreateForwardRuleArgs) {
        let etl_rule = match parse_etl_rule(&args.etl_rule_json) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("invalid --etl-rule-json: {e}");
                return;
            }
        };
        let req = CreateForwardRuleReq {
            tenant: args.tenant,
            rule_name: args.rule_name,
            topic_name: args.topic_name,
            mail_address_prefixes: args.mail_address_prefix,
            any_tags: args.any_tag,
            priorities: args.priority,
            sender_prefixes: args.sender_prefix,
            keep_headers: args.keep_headers,
            on_failure: args.on_failure,
            enabled: args.enabled,
            etl_rule,
        };
        match self.client().create_mq9_forward_rule(&req).await {
            Ok(_) => println!("Created successfully!"),
            Err(e) => eprintln!("create forward rule failed: {e}"),
        }
    }

    async fn update_forward_rule(&self, args: UpdateForwardRuleArgs) {
        let etl_rule = match parse_etl_rule(&args.etl_rule_json) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("invalid --etl-rule-json: {e}");
                return;
            }
        };
        let req = UpdateForwardRuleReq {
            tenant: args.tenant,
            rule_name: args.rule_name,
            topic_name: args.topic_name,
            mail_address_prefixes: args.mail_address_prefix,
            any_tags: args.any_tag,
            priorities: args.priority,
            sender_prefixes: args.sender_prefix,
            keep_headers: args.keep_headers,
            on_failure: args.on_failure,
            enabled: args.enabled,
            etl_rule,
        };
        match self.client().update_mq9_forward_rule(&req).await {
            Ok(_) => println!("Updated successfully!"),
            Err(e) => eprintln!("update forward rule failed: {e}"),
        }
    }

    async fn delete_forward_rule(&self, args: DeleteForwardRuleArgs) {
        let req = DeleteForwardRuleReq {
            tenant: args.tenant,
            rule_name: args.rule_name,
        };
        match self.client().delete_mq9_forward_rule(&req).await {
            Ok(_) => println!("Deleted successfully!"),
            Err(e) => eprintln!("delete forward rule failed: {e}"),
        }
    }

    async fn toggle_forward_rule(&self, args: ToggleForwardRuleArgs, enabled: bool) {
        let detail_req = ForwardRuleDetailReq {
            tenant: args.tenant.clone(),
            rule_name: args.rule_name.clone(),
        };
        let row = match self
            .client()
            .get_mq9_forward_rule_detail::<_, ForwardRuleListRow>(&detail_req)
            .await
        {
            Ok(row) => row,
            Err(e) => {
                eprintln!("fetch forward rule failed: {e}");
                return;
            }
        };
        let req = UpdateForwardRuleReq {
            tenant: row.tenant,
            rule_name: row.rule_name,
            topic_name: row.topic_name,
            mail_address_prefixes: row.mail_address_prefixes,
            any_tags: row.any_tags,
            priorities: row.priorities,
            sender_prefixes: row.sender_prefixes,
            keep_headers: row.keep_headers,
            on_failure: Some(row.on_failure),
            enabled,
            etl_rule: row.etl_rule,
        };
        match self.client().update_mq9_forward_rule(&req).await {
            Ok(_) => println!(
                "{} successfully!",
                if enabled { "Enabled" } else { "Disabled" }
            ),
            Err(e) => eprintln!("toggle forward rule failed: {e}"),
        }
    }
}

fn parse_etl_rule(json: &Option<String>) -> Result<Option<ETLRule>, String> {
    match json {
        None => Ok(None),
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => serde_json::from_str(s).map(Some).map_err(|e| e.to_string()),
    }
}
