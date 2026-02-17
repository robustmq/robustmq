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

pub mod common;
pub mod conn;
pub mod publish;
pub mod report;
pub mod stats;
pub mod subscribe;

use crate::error::BenchMarkError;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
pub struct MqttBenchArgs {
    #[command(subcommand)]
    pub command: MqttBenchCommand,
}

#[derive(Debug, Subcommand)]
pub enum MqttBenchCommand {
    Conn(ConnBenchArgs),
    Pub(PublishBenchArgs),
    Sub(SubscribeBenchArgs),
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

#[derive(Debug, Clone, Parser)]
pub struct CommonMqttBenchArgs {
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    pub host: String,
    #[arg(long, default_value_t = 1883)]
    pub port: u16,
    #[arg(long, default_value_t = 1000)]
    pub count: usize,
    #[arg(long, default_value_t = 0)]
    pub interval_ms: u64,
    #[arg(long, default_value_t = 0)]
    pub qos: u8,
    #[arg(long)]
    pub username: Option<String>,
    #[arg(long)]
    pub password: Option<String>,
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub output: OutputFormat,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ConnMode {
    Create,
    Hold,
}

#[derive(Debug, Clone, Parser)]
pub struct ConnBenchArgs {
    #[command(flatten)]
    pub common: CommonMqttBenchArgs,
    #[arg(long, default_value_t = 1000)]
    pub concurrency: usize,
    #[arg(long, value_enum, default_value_t = ConnMode::Create)]
    pub mode: ConnMode,
    #[arg(long, default_value_t = 60)]
    pub hold_secs: u64,
}

#[derive(Debug, Clone, Parser)]
pub struct PublishBenchArgs {
    #[command(flatten)]
    pub common: CommonMqttBenchArgs,
    #[arg(long, default_value_t = String::from("bench/%i"))]
    pub topic: String,
    #[arg(long, default_value_t = 256)]
    pub payload_size: usize,
    #[arg(long, default_value_t = 1000)]
    pub message_interval_ms: u64,
    #[arg(long, default_value_t = 60)]
    pub duration_secs: u64,
}

#[derive(Debug, Clone, Parser)]
pub struct SubscribeBenchArgs {
    #[command(flatten)]
    pub common: CommonMqttBenchArgs,
    #[arg(long, default_value_t = String::from("bench/#"))]
    pub topic: String,
    #[arg(long, default_value_t = 60)]
    pub duration_secs: u64,
}

pub fn handle_mqtt_bench(args: MqttBenchArgs) -> Result<(), BenchMarkError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(BenchMarkError::IoError)?;

    runtime.block_on(async move {
        match args.command {
            MqttBenchCommand::Conn(args) => conn::run_conn_bench(args).await,
            MqttBenchCommand::Pub(args) => publish::run_publish_bench(args).await,
            MqttBenchCommand::Sub(args) => subscribe::run_subscribe_bench(args).await,
        }
    })
}
