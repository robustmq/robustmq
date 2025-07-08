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

pub mod logo;
use logo::DEFAULT_PLACEMENT_CENTER_CONFIG;
use std::sync::OnceLock;
use tracing::{error, info};

use crate::tools::read_file;

// Static variable to hold the version in memory
static VERSION: OnceLock<String> = OnceLock::new();

/// Returns the RobustMQ version
///
/// On first call, reads version from config/version.ini file and caches it in memory.
/// Subsequent calls return the cached version without file I/O.
///
/// # Return value
/// Returns the version string. If version file cannot be read, returns the hardcoded version
/// or a placeholder string.
pub fn version() -> String {
    // Return cached version if already loaded
    if let Some(cached_version) = VERSION.get() {
        return cached_version.clone();
    }

    // Try to read version from file
    let version_str = match read_file(DEFAULT_PLACEMENT_CENTER_CONFIG) {
        Ok(data) => {
            // Trim whitespace and newlines
            let version = data.trim().to_string();
            info!("Read version from file: {}", version);
            version
        }
        Err(e) => {
            error!("Failed to read version file: {}", e);
            // Fallback to workspace version from Cargo.toml
            let fallback = option_env!("CARGO_PKG_VERSION").unwrap_or("unknown");
            error!("Using fallback version: {}", fallback);
            fallback.to_string()
        }
    };

    // Cache the version in memory
    match VERSION.set(version_str.clone()) {
        Ok(_) => info!("Version cached in memory"),
        Err(_) => error!("Failed to cache version in memory (race condition)"),
    }

    version_str
}
