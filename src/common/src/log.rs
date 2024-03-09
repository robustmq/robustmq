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

use log::LevelFilter;
use log4rs::{
    append::{
        console::ConsoleAppender,
        rolling_file::{
            policy::compound::{
                roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
            },
            RollingFileAppender,
        },
    },
    config::{Appender, Logger, Root},
    encode::pattern::PatternEncoder,
    Config,
};

use crate::config::placement_center::{placement_center_conf, PlacementCenterConfig};

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

pub fn debug_meta(msg: &str) -> () {
    log::debug!(target:"app::meta", "{}",msg)
}

pub fn error_meta(msg: &str) -> () {
    log::error!(target:"app::meta", "{}",msg)
}

pub fn init_log(path: String, segment_log_size: u64, log_fie_count: u32) {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} {h({l})} {m}{n}",
        )))
        .build();

    let server_log = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} {h({l})} {m}{n}",
        )))
        .append(true)
        .build(
            format!("{}/server.log", path),
            Box::new(CompoundPolicy::new(
                Box::new(SizeTrigger::new(segment_log_size)),
                Box::new(
                    FixedWindowRoller::builder()
                        .base(0)
                        .build(&format!("{}/server.{}.log", path, "{}"), log_fie_count)
                        .unwrap(),
                ),
            )),
        )
        .unwrap();

    let meta_log = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} {h({l})} {m}{n}",
        )))
        .append(true)
        .build(
            format!("{}/meta.log", path),
            Box::new(CompoundPolicy::new(
                Box::new(SizeTrigger::new(segment_log_size)),
                Box::new(
                    FixedWindowRoller::builder()
                        .base(0)
                        .build(&format!("{}/meta.{}.log", path, "{}"), log_fie_count)
                        .unwrap(),
                ),
            )),
        )
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("server", Box::new(server_log)))
        .appender(Appender::builder().build("meta", Box::new(meta_log)))
        .logger(
            Logger::builder()
                .appender("server")
                .appender("stdout")
                .additive(false)
                .build("app::server", LevelFilter::Info),
        )
        .logger(
            Logger::builder()
                .appender("meta")
                .appender("stdout")
                .additive(false)
                .build("app::meta", LevelFilter::Info),
        )
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();

    let _ = log4rs::init_config(config).unwrap();
}

pub fn init_placement_center_log() {
    let conf = placement_center_conf();
    init_log(
        conf.log_path.clone(),
        conf.log_segment_size,
        conf.log_file_num,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_print() {
        init_log("".to_string(), 1024 * 1024 * 1024, 50);
        info("lobo");
        info_meta("server lobo");
    }
}
