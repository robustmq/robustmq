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

pub(crate) mod mqtt;

use clap::{arg, Parser, Subcommand, ValueEnum};
use cli_command::mqtt::{MqttActionType, MqttBrokerCommand, MqttCliCommandParam};
use cli_command::placement::{
    PlacementActionType, PlacementCenterCommand, PlacementCliCommandParam,
};
use protocol::broker_mqtt::broker_mqtt_admin::{CreateUserRequest, DeleteUserRequest, EnableConnectionJitterRequest,  ListTopicRequest};
use protocol::placement_center::placement_center_openraft::{
    AddLearnerRequest, ChangeMembershipRequest, Node,
};

use crate::mqtt::admin::{
    process_slow_sub_args, CreateUserArgs, DeleteUserArgs, ConnectionJitterArgs, SlowSubArgs,
};

#[derive(Parser)] // requires `derive` feature
#[command(name = "robust-ctl")]
#[command(bin_name = "robust-ctl")]
#[command(styles = CLAP_STYLING)]
#[command(author="RobustMQ", version="0.0.1", about="Command line tool for RobustMQ", long_about = None)]
#[command(next_line_help = true)]
enum RobustMQCli {
    Mqtt(MqttArgs),
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
struct MqttArgs {
    #[arg(short, long,default_value_t =String::from("127.0.0.1:9981"))]
    server: String,

    #[clap(subcommand)]
    action: MQTTAction,
}

#[derive(Debug, Subcommand)]
enum MQTTAction {
    Status,
    // User admin
    CreateUser(CreateUserArgs),
    DeleteUser(DeleteUserArgs),
    ListUser,

    // Connections
    ListConnection,

    // connection jitter
    #[clap(name = "connection-jitter")]
    ConnectionJitter(ConnectionJitterArgs),

    ListTopic(ListTopicArgs),

    // observability: slow-sub feat
    #[clap(name = "slow-sub")]
    SlowSub(SlowSubArgs),
}

#[derive(ValueEnum, Clone, Debug)]
enum MatchOption {
    E,
    P,
    S,
}

#[derive(clap::Args, Debug)]
#[command(author="RobustMQ", about="action: list topics", long_about = None)]
#[command(next_line_help = true)]
struct ListTopicArgs {
    #[arg(short, long, required = true)]
    topic_name: String,

    #[arg(short, long, default_value = "e")]
    match_option: MatchOption,
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
        RobustMQCli::Mqtt(args) => handle_mqtt(args, MqttBrokerCommand::new()).await,
        RobustMQCli::Place(args) => handle_placement(args, PlacementCenterCommand::new()).await,
        RobustMQCli::Journal(args) => handle_journal(args).await,
    }
}

async fn handle_mqtt(args: MqttArgs, cmd: MqttBrokerCommand) {
    let params = MqttCliCommandParam {
        server: args.server,
        action: match args.action {
            MQTTAction::Status => MqttActionType::Status,
            MQTTAction::CreateUser(arg) => MqttActionType::CreateUser(CreateUserRequest {
                username: arg.username,
                password: arg.password,
                is_superuser: arg.is_superuser,
            }),
            MQTTAction::DeleteUser(arg) => MqttActionType::DeleteUser(DeleteUserRequest {
                username: arg.username,
            }),
            MQTTAction::ListUser => MqttActionType::ListUser,
            MQTTAction::ListConnection => MqttActionType::ListConnection,
            MQTTAction::ListTopic(args) => MqttActionType::ListTopic(ListTopicRequest {
                topic_name: args.topic_name,
                match_option: match args.match_option {
                    MatchOption::E => 0,
                    MatchOption::P => 1,
                    MatchOption::S => 2,
                },
            }),
            MQTTAction::SlowSub(args) => process_slow_sub_args(args),
            MQTTAction::ConnectionJitter(args) => {
                MqttActionType::EnableConnectionJitter(EnableConnectionJitterRequest {
                    is_enable: args.is_enable.unwrap_or(false),
                    window_time: args.window_time.unwrap_or(1),
                    max_client_connections: args.max_client_connections.unwrap_or(15),
                    ban_time: args.ban_time.unwrap_or(5),
                })
            }
        },
    };
    cmd.start(params).await;
}

async fn handle_placement(args: PlacementArgs, cmd: PlacementCenterCommand) {
    let params = PlacementCliCommandParam {
        server: args.server,
        action: match args.action {
            PlacementAction::Status => PlacementActionType::Status,
            PlacementAction::AddLearner(arg) => {
                PlacementActionType::AddLearner(AddLearnerRequest {
                    node_id: arg.node_id,
                    node: Some(Node {
                        node_id: arg.node_id,
                        rpc_addr: arg.rpc_addr,
                    }),
                    blocking: arg.blocking,
                })
            }
            PlacementAction::ChangeMembership(arg) => {
                PlacementActionType::ChangeMembership(ChangeMembershipRequest {
                    members: arg.members,
                    retain: arg.retain,
                })
            }
        },
    };
    cmd.start(params).await;
}

// TODO: implement journal engine
async fn handle_journal(args: JournalArgs) {
    println!("{:?}", args);
}
