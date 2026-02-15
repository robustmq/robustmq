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

pub mod meta;

use crate::error::BenchMarkError;
use crate::mqtt::OutputFormat;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct MetaBenchArgs {
    #[command(subcommand)]
    pub command: MetaBenchCommand,
}

#[derive(Debug, Subcommand)]
pub enum MetaBenchCommand {
    PlacementCreateSession(PlacementCreateSessionArgs),
}

#[derive(Debug, Clone, Parser)]
pub struct PlacementCreateSessionArgs {
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    pub host: String,
    #[arg(long, default_value_t = 1228)]
    pub port: u16,
    #[arg(long, default_value_t = 10000)]
    pub count: usize,
    #[arg(long, default_value_t = 1000)]
    pub concurrency: usize,
    #[arg(long, default_value_t = 3000)]
    pub timeout_ms: u64,
    #[arg(long, default_value_t = 3600)]
    pub session_expiry_secs: u64,
    #[arg(long, default_value_t = String::from("bench-meta-session"))]
    pub client_id_prefix: String,
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub output: OutputFormat,
}

pub fn handle_meta_bench(args: MetaBenchArgs) -> Result<(), BenchMarkError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(BenchMarkError::IoError)?;

    runtime.block_on(async move {
        match args.command {
            MetaBenchCommand::PlacementCreateSession(params) => {
                meta::run_placement_create_session_bench(params).await
            }
        }
    })
}
