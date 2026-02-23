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

use sysinfo::{Pid, ProcessExt, System, SystemExt};

pub fn total_memory() -> u64 {
    let mut system = System::new_all();
    system.refresh_memory();
    system.total_memory()
}

pub fn used_memory() -> u64 {
    let mut system = System::new_all();
    system.refresh_memory();
    system.used_memory()
}

pub fn system_memory_usage() -> f32 {
    let mut system = System::new_all();
    system.refresh_memory();
    let total = system.total_memory();
    if total == 0 {
        return 0.0;
    }
    (system.used_memory() as f32 / total as f32) * 100.0
}

/// Returns RSS (Resident Set Size) of the current process in bytes.
pub fn process_memory() -> u64 {
    let mut system = System::new_all();
    let pid = Pid::from(std::process::id() as usize);
    system.refresh_memory();
    system.refresh_processes();
    system.process(pid).map(|p| p.memory()).unwrap_or(0)
}

pub fn process_memory_usage() -> f32 {
    let mut system = System::new_all();
    let pid = Pid::from(std::process::id() as usize);
    system.refresh_memory();
    system.refresh_processes();

    let total = system.total_memory();
    if total == 0 {
        return 0.0;
    }
    system
        .process(pid)
        .map(|p| (p.memory() as f32 / total as f32) * 100.0)
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_memory() {
        let total = total_memory();
        assert!(total > 0, "total_memory should be > 0, got {total}");
    }

    #[test]
    fn test_used_memory() {
        let total = total_memory();
        let used = used_memory();
        assert!(used > 0, "used_memory should be > 0, got {used}");
        assert!(
            used <= total,
            "used_memory {used} should not exceed total {total}"
        );
    }

    #[test]
    fn test_system_memory_usage() {
        let usage = system_memory_usage();
        assert!(
            (0.0..=100.0).contains(&usage),
            "system_memory_usage should be in [0, 100], got {usage}"
        );
    }

    #[test]
    fn test_process_memory() {
        let rss = process_memory();
        assert!(rss > 0, "process_memory (RSS) should be > 0, got {rss}");
    }

    #[test]
    fn test_process_memory_usage() {
        let usage = process_memory_usage();
        assert!(
            (0.0..=100.0).contains(&usage),
            "process_memory_usage should be in [0, 100], got {usage}"
        );
    }
}
