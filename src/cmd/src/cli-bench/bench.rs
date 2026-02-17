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

use clap::Parser;
use cli_bench::{
    grpc::handle_meta_bench, mqtt::handle_mqtt_bench, BenchMarkError, RobustMQBench,
    RobustMQBenchCommand,
};

fn main() -> Result<(), BenchMarkError> {
    let args = RobustMQBench::parse();
    match args.command {
        RobustMQBenchCommand::Meta(meta_args) => handle_meta_bench(meta_args)?,
        RobustMQBenchCommand::Mqtt(mqtt_args) => handle_mqtt_bench(mqtt_args)?,
    }

    Ok(())
}
