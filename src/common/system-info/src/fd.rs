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

/// On non-Linux platforms returns 0.
pub fn process_fd_count() -> u64 {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_dir("/proc/self/fd")
            .map(|entries| entries.count() as u64)
            .unwrap_or(0)
    }
    #[cfg(not(target_os = "linux"))]
    {
        0
    }
}

/// Returns `(current_open, max_allowed)` system-wide file descriptors.
///
/// Reads `/proc/sys/fs/file-nr` which contains `allocated  unused  max`.
/// `current_open` = allocated − unused.
/// On non-Linux platforms both values are 0.
pub fn system_fd_count() -> (u64, u64) {
    #[cfg(target_os = "linux")]
    {
        match std::fs::read_to_string("/proc/sys/fs/file-nr") {
            Ok(content) => {
                let parts: Vec<u64> = content
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if parts.len() >= 3 {
                    let current = parts[0].saturating_sub(parts[1]);
                    let max = parts[2];
                    return (current, max);
                }
                (0, 0)
            }
            Err(_) => (0, 0),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        (0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_fd_count() {
        let count = process_fd_count();
        #[cfg(target_os = "linux")]
        assert!(
            count > 0,
            "process_fd_count should be > 0 on Linux, got {count}"
        );
        #[cfg(not(target_os = "linux"))]
        assert_eq!(count, 0, "process_fd_count should be 0 on non-Linux");
    }

    #[test]
    fn test_system_fd_count() {
        let (current, max) = system_fd_count();
        #[cfg(target_os = "linux")]
        {
            assert!(max > 0, "system fd max should be > 0 on Linux, got {max}");
            assert!(
                current <= max,
                "current {current} should not exceed max {max}"
            );
        }
        #[cfg(not(target_os = "linux"))]
        assert_eq!((current, max), (0, 0), "should be (0, 0) on non-Linux");
    }
}
