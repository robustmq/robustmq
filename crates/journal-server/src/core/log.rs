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

use common_base::{error::log_config::LogConfigError, logging::init_tracing_subscriber};
use common_config::broker::broker_config;
use tracing_appender::non_blocking::WorkerGuard;

pub fn init_journal_server_log() -> Result<Vec<WorkerGuard>, LogConfigError> {
    let conf = broker_config();
    init_tracing_subscriber(&conf.log.log_config, &conf.log.log_path)
}
