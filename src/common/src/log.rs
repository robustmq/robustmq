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

pub const DEFAULT_LOG_CONFIG: &str = "config/log4rs.yml";

pub fn info(msg: &str) -> () {
    log::info!(target:"app::server", "{}",msg)
}

pub fn debug(msg: &str) -> () {
    log::debug!(target:"app::server", "{}",msg)
}

pub fn error(msg: &str) -> () {
    log::error!(target:"app::server", "{}",msg)
}

pub fn info_meta(msg: &str) -> () {
    log::info!(target:"app::meta", "{}",msg)
}

pub fn error_meta(msg: &str) -> () {
    log::error!(target:"app::meta", "{}",msg)
}

pub fn new(){
    log4rs::init_file(DEFAULT_LOG_CONFIG, Default::default()).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_print() {
        log4rs::init_file(format!("../../../{}", DEFAULT_LOG_CONFIG), Default::default()).unwrap();
        info("lobo");
        info_meta("server lobo");
    }
}