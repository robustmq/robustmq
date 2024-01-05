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

use clap::command;
use clap::Parser;
use common::config::meta::MetaConfig;
use common::config::parse_meta;
use common::config::DEFAULT_META_CONFIG;
use common::log;
use meta::Meta;

#[derive(Parser, Debug)]
#[command(author="robustmq", version="0.0.1", about=" RobustMQ: Next generation cloud-native converged high-performance message queue.", long_about = None)]
#[command(next_line_help = true)]

struct ArgsParams {
    /// MetaService Indicates the path of the configuration file
    #[arg(short, long, default_value_t=String::from(DEFAULT_META_CONFIG))]
    meta_conf: String,
}

fn main() {
    let args = ArgsParams::parse();
    let meta_conf: MetaConfig = parse_meta(&args.meta_conf);
    log::new(
        meta_conf.log_path.clone(),
        meta_conf.log_segment_size.clone(),
        meta_conf.log_file_num.clone(),
    );
    let mut mt_s = Meta::new(meta_conf);
    mt_s.start();
}
