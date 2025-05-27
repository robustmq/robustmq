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

pub mod get;
pub mod mixed;
pub mod set;

use crate::{error::BenchMarkError, BenchMark};
use clap::{Parser, Subcommand};
use common_base::runtime::create_runtime;
use get::KvGetBenchArgs;
use mixed::KvMixedBenchArgs;
use set::KvSetBenchArgs;

#[derive(Debug, Clone, Subcommand)]
pub enum KvBenchAction {
    Get(KvGetBenchArgs),
    Set(KvSetBenchArgs),
    Mixed(KvMixedBenchArgs),
}

#[derive(Debug, Parser)]
pub struct KvBenchArgs {
    #[command(subcommand)]
    pub action: KvBenchAction,
}

pub fn handle_kv_bench(args: KvBenchArgs) -> Result<(), BenchMarkError> {
    let (num_threads, thread_name) = match args.action {
        KvBenchAction::Get(ref get_args) => (get_args.worker_threads, "bench-kv-get"),
        KvBenchAction::Set(_) => unimplemented!(),
        KvBenchAction::Mixed(_) => unimplemented!(),
    };

    let rt = create_runtime(thread_name, num_threads);

    // Run client_write benchmark
    match args.action {
        KvBenchAction::Get(get_args) => {
            rt.block_on(get_args.do_bench())?;
        }
        KvBenchAction::Set(_) => {
            unimplemented!();
        }
        KvBenchAction::Mixed(_) => {
            unimplemented!();
        }
    }

    Ok(())
}
