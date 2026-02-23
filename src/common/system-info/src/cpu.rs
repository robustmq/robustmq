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

use sysinfo::{CpuExt, Pid, ProcessExt, System, SystemExt};
use tokio::time::{sleep, Duration};

pub fn cpu_count() -> usize {
    let mut system = System::new_all();
    system.refresh_cpu();
    system.cpus().len()
}

/// CPU usage is normalized by core count so the result stays within 0.0–100.0
/// even on multi-core systems (sysinfo reports raw per-core sum otherwise).
/// Two samples 200 ms apart are required to compute a meaningful delta.
pub async fn process_cpu_usage() -> f32 {
    let mut system = System::new_all();
    let pid = Pid::from(std::process::id() as usize);

    system.refresh_cpu();
    system.refresh_processes();
    sleep(Duration::from_millis(200)).await;
    system.refresh_cpu();
    system.refresh_processes();

    if let Some(process) = system.process(pid) {
        let cpu_count = system.cpus().len() as f32;
        if cpu_count > 0.0 {
            return process.cpu_usage() / cpu_count;
        }
    }
    0.0
}

/// Two samples 200 ms apart are required to compute a meaningful delta.
pub async fn system_cpu_usage() -> f32 {
    let mut system = System::new_all();
    system.refresh_cpu();
    sleep(Duration::from_millis(200)).await;
    system.refresh_cpu();
    system.global_cpu_info().cpu_usage()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_count() {
        let count = cpu_count();
        assert!(count > 0, "cpu_count should be at least 1, got {count}");
    }

    #[tokio::test]
    async fn test_process_cpu_usage() {
        let usage = process_cpu_usage().await;
        assert!(
            (0.0..=100.0).contains(&usage),
            "process_cpu_usage should be in [0, 100], got {usage}"
        );
    }

    #[tokio::test]
    async fn test_system_cpu_usage() {
        let usage = system_cpu_usage().await;
        assert!(
            (0.0..=100.0).contains(&usage),
            "system_cpu_usage should be in [0, 100], got {usage}"
        );
    }
}
