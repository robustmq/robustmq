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
use cli_command::mqtt::{
    CreateUserCliRequest, DeleteUserCliRequest, MqttActionType, MqttBrokerCommand,
    MqttCliCommandParam,
};
use cli_command::placement::{
    AddLearnerCliRequset, ChangeMembershipCliRequest, PlacementActionType, PlacementCenterCommand,
    PlacementCliCommandParam,
};

#[derive(Parser)] // requires `derive` feature
#[command(name = "robust-ctl")]
#[command(bin_name = "robust-ctl")]
#[command(styles = CLAP_STYLING)]
#[command(author="RobustMQ", version="0.0.1", about="Command line tool for RobustMQ", long_about = None)]
#[command(next_line_help = true)]
enum RobustMQCli {
    Mqtt(MQTTArgs),
    Place(PlacementArgs),
    Journal(JournalArgs),
}

pub const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(clap_cargo::style::HEADER)
    .usage(clap_cargo::style::USAGE)
    .literal(clap_cargo::style::LITERAL)
    .placeholder(clap_cargo::style::PLACEHOLDER)
    .error(clap_cargo::style::ERROR)
    .valid(clap_cargo::style::VALID)
    .invalid(clap_cargo::style::INVALID);

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ", about="Command line tool for mqtt broker", long_about = None)]
#[command(next_line_help = true)]
struct MQTTArgs {
    #[arg(short, long,default_value_t =String::from("127.0.0.1:9981"))]
    server: String,

    #[clap(subcommand)]
    action: MQTTAction,
}

#[derive(Debug, Subcommand)]
enum MQTTAction {
    Status,
    CreateUser(CreateUserArgs),
    DeleteUser(DeleteUserArgs),
    ListUser,
}

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ", about="action: create user", long_about = None)]
#[command(next_line_help = true)]
struct CreateUserArgs {
    #[arg(short, long, required = true)]
    username: String,

    #[arg(short, long, required = true)]
    password: String,

    #[arg(short, long, default_value_t = false)]
    is_superuser: bool,
}

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ", about="action: delete user", long_about = None)]
#[command(next_line_help = true)]
struct DeleteUserArgs {
    #[arg(short, long, required = true)]
    username: String,
}

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ",  about="Command line tool for placement center", long_about = None)]
#[command(next_line_help = true)]
struct PlacementArgs {
    #[arg(short, long, default_value_t = String::from("127.0.0.1:1228"))]
    server: String,

    #[clap(subcommand)]
    action: PlacementAction,
}

#[derive(Debug, Subcommand)]
enum PlacementAction {
    Status,
    AddLearner(AddLearnerArgs),
    ChangeMembership(ChangeMembershipArgs),
}

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ", about="action: add learner", long_about = None)]
#[command(next_line_help = true)]
struct AddLearnerArgs {
    #[arg(short, long, required = true)]
    node_id: u64,

    #[arg(short, long, default_value_t = String::from("127.0.0.1:1228"))]
    rpc_addr: String,

    #[arg(short, long, default_value_t = true)]
    blocking: bool,
}

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ",  about="action: change membership", long_about = None)]
#[command(next_line_help = true)]
struct ChangeMembershipArgs {
    #[arg(short, long, num_args = 1.., required = true)]
    members: Vec<u64>,

    #[arg(short, long, default_value_t = true)]
    retain: bool,
}

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ", about="Command line tool for journal engine", long_about = None)]
#[command(next_line_help = true)]
struct JournalArgs {
    #[arg(short, long,default_value_t =String::from("127.0.0.1:1228"))]
    server: String,

    #[arg(short, long,default_value_t =String::from("status"))]
    action: String,
}

#[tokio::main]
async fn main() {
    let args = RobustMQCli::parse();
    match args {
        RobustMQCli::Mqtt(args) => {
            let cmd = MqttBrokerCommand::new();
            let params = MqttCliCommandParam {
                server: args.server,
                action: match args.action {
                    MQTTAction::Status => MqttActionType::Status,
                    MQTTAction::CreateUser(arg) => MqttActionType::CreateUser(
                        CreateUserCliRequest::new(arg.username, arg.password, arg.is_superuser),
                    ),
                    MQTTAction::DeleteUser(arg) => {
                        MqttActionType::DeleteUser(DeleteUserCliRequest::new(arg.username))
                    }
                    MQTTAction::ListUser => MqttActionType::ListUser,
                },
            };
            cmd.start(params).await;
        }
        RobustMQCli::Place(args) => {
            let cmd = PlacementCenterCommand::new();
            let params = PlacementCliCommandParam {
                server: args.server,
                action: match args.action {
                    PlacementAction::Status => PlacementActionType::Status,
                    PlacementAction::AddLearner(arg) => PlacementActionType::AddLearner(
                        AddLearnerCliRequset::new(arg.node_id, arg.rpc_addr, arg.blocking),
                    ),
                    PlacementAction::ChangeMembership(arg) => {
                        PlacementActionType::ChangeMembership(ChangeMembershipCliRequest::new(
                            arg.members,
                            arg.retain,
                        ))
                    }
                },
            };
            cmd.start(params).await;
        }
        RobustMQCli::Journal(args) => {
            println!("{:?}", args);
        }
    }
}
