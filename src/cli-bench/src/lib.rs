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

pub mod error;
pub mod grpc;
pub mod mqtt;

use crate::grpc::MetaBenchArgs;
use crate::mqtt::MqttBenchArgs;
use clap::{Parser, Subcommand};
pub use error::BenchMarkError;

#[derive(Parser)]
#[command(name = "robust-bench")]
#[command(bin_name = "robust-bench")]
#[command(author="RobustMQ", version="0.0.1", about="Benchmark tool for RobustMQ", long_about = None)]
#[command(next_line_help = true)]
pub struct RobustMQBench {
    #[command(subcommand)]
    pub command: RobustMQBenchCommand,
}

#[derive(Debug, Subcommand)]
pub enum RobustMQBenchCommand {
    Meta(MetaBenchArgs),
    Mqtt(MqttBenchArgs),
}

#[async_trait::async_trait]
pub trait BenchMark {
    fn validate(&self) -> Result<(), BenchMarkError>;
    async fn do_bench(&self) -> Result<(), BenchMarkError>;
}
