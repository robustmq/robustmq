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

use common_base::port::is_local_port_listening;
use std::thread::sleep;
use std::time::{Duration, Instant};
use tracing::{error, info};

pub fn wait_for_port_ready(
    port: u32,
    service_name: &str,
    max_wait_time: Duration,
    check_interval: Duration,
) -> bool {
    let start_time = Instant::now();
    let mut checks = 0u32;

    while start_time.elapsed() < max_wait_time {
        if is_local_port_listening(port) {
            info!("{service_name} startup check completed on port {port}");
            return true;
        }

        checks += 1;
        if checks.is_multiple_of(10) {
            info!(
                "{service_name} not ready yet on port {} (elapsed: {:?})",
                port,
                start_time.elapsed()
            );
        }

        sleep(check_interval);
    }

    error!("{service_name} failed to start within {:?}", max_wait_time);
    false
}

pub fn wait_for_grpc_ready(grpc_port: u32) -> bool {
    wait_for_port_ready(
        grpc_port,
        "GRPC server",
        Duration::from_secs(10),
        Duration::from_millis(100),
    )
}

pub fn wait_for_engine_ready(engine_port: u32) -> bool {
    wait_for_port_ready(
        engine_port,
        "Storage Engine",
        Duration::from_secs(10),
        Duration::from_millis(100),
    )
}
