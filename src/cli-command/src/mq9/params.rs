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

use clap::{Parser, Subcommand};

/// `robust-ctl mq9` — manage mq9 protocol resources.
#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "mq9 management commands", long_about = None)]
#[command(next_line_help = true)]
pub struct Mq9Args {
    #[arg(short, long, default_value_t = String::from("127.0.0.1:8080"))]
    pub server: String,
    #[arg(long, default_value_t = 1)]
    pub page: u32,
    #[arg(long, default_value_t = 100)]
    pub limit: u32,
    #[command(subcommand)]
    pub action: Mq9Action,
}

#[derive(Debug, Subcommand)]
pub enum Mq9Action {
    /// Manage Mq9 forward rules (broker-side fan-out rules).
    ForwardRule(ForwardRuleArgs),
}

#[derive(clap::Args, Debug)]
#[command(author = "RobustMQ", about = "Manage mq9 forward rules", long_about = None)]
#[command(next_line_help = true)]
pub struct ForwardRuleArgs {
    #[command(subcommand)]
    pub action: ForwardRuleAction,
}

#[derive(Debug, Subcommand)]
pub enum ForwardRuleAction {
    /// List forward rules (optional tenant/rule_name filter).
    List(ListForwardRuleArgs),
    /// Show one forward rule.
    Detail(DetailForwardRuleArgs),
    /// Create a forward rule.
    Create(CreateForwardRuleArgs),
    /// Update an existing forward rule (full replace).
    Update(UpdateForwardRuleArgs),
    /// Delete a forward rule.
    Delete(DeleteForwardRuleArgs),
    /// Enable a forward rule (shortcut for update --enabled true).
    Enable(ToggleForwardRuleArgs),
    /// Disable a forward rule (shortcut for update --enabled false).
    Disable(ToggleForwardRuleArgs),
}

#[derive(Parser, Debug, Clone)]
#[command(next_line_help = true)]
pub struct ListForwardRuleArgs {
    #[arg(short = 'T', long)]
    pub tenant: Option<String>,
    #[arg(short = 'n', long)]
    pub rule_name: Option<String>,
    #[arg(short = 't', long)]
    pub topic_name: Option<String>,
    #[arg(long)]
    pub enabled: Option<bool>,
}

#[derive(Parser, Debug, Clone)]
#[command(next_line_help = true)]
pub struct DetailForwardRuleArgs {
    #[arg(short = 'T', long, required = true)]
    pub tenant: String,
    #[arg(short = 'n', long, required = true)]
    pub rule_name: String,
}

#[derive(Parser, Debug, Clone)]
#[command(next_line_help = true)]
pub struct CreateForwardRuleArgs {
    #[arg(short = 'T', long, required = true)]
    pub tenant: String,
    #[arg(short = 'n', long, required = true)]
    pub rule_name: String,
    /// Destination broker topic.
    #[arg(short = 't', long, required = true)]
    pub topic_name: String,
    /// Match mailbox address prefixes. Repeatable. `*` or empty = wildcard.
    #[arg(long = "mail-prefix")]
    pub mail_address_prefix: Vec<String>,
    /// Match any of these user tags. Repeatable.
    #[arg(long = "tag")]
    pub any_tag: Vec<String>,
    /// Match these priorities (normal|urgent|critical). Repeatable.
    #[arg(long = "priority")]
    pub priority: Vec<String>,
    /// Match sender mailbox address prefixes. Repeatable.
    #[arg(long = "sender-prefix")]
    pub sender_prefix: Vec<String>,
    /// Forward original message headers (default: true).
    #[arg(long, default_value_t = true)]
    pub keep_headers: bool,
    /// Failure strategy: `drop_and_log` (default) or `fail_send`.
    #[arg(long)]
    pub on_failure: Option<String>,
    /// Enable the rule immediately (default: true).
    #[arg(long, default_value_t = true)]
    pub enabled: bool,
    /// Optional inline ETL rule JSON applied to the forked payload.
    #[arg(long)]
    pub etl_rule_json: Option<String>,
}

#[derive(Parser, Debug, Clone)]
#[command(next_line_help = true)]
pub struct UpdateForwardRuleArgs {
    #[arg(short = 'T', long, required = true)]
    pub tenant: String,
    #[arg(short = 'n', long, required = true)]
    pub rule_name: String,
    #[arg(short = 't', long, required = true)]
    pub topic_name: String,
    #[arg(long = "mail-prefix")]
    pub mail_address_prefix: Vec<String>,
    #[arg(long = "tag")]
    pub any_tag: Vec<String>,
    #[arg(long = "priority")]
    pub priority: Vec<String>,
    #[arg(long = "sender-prefix")]
    pub sender_prefix: Vec<String>,
    #[arg(long, default_value_t = true)]
    pub keep_headers: bool,
    #[arg(long)]
    pub on_failure: Option<String>,
    #[arg(long, default_value_t = true)]
    pub enabled: bool,
    #[arg(long)]
    pub etl_rule_json: Option<String>,
}

#[derive(Parser, Debug, Clone)]
#[command(next_line_help = true)]
pub struct DeleteForwardRuleArgs {
    #[arg(short = 'T', long, required = true)]
    pub tenant: String,
    #[arg(short = 'n', long, required = true)]
    pub rule_name: String,
}

#[derive(Parser, Debug, Clone)]
#[command(next_line_help = true)]
pub struct ToggleForwardRuleArgs {
    #[arg(short = 'T', long, required = true)]
    pub tenant: String,
    #[arg(short = 'n', long, required = true)]
    pub rule_name: String,
}
