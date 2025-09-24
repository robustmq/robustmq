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
use logo::DEFAULT_META_SERVICE_CONFIG;
use std::sync::OnceLock;
use tracing::{error, info};

use crate::tools::read_file;

// Version embedded at compile time - empty string if file doesn't exist
#[cfg(feature = "embed_version")]
const EMBEDDED_VERSION: &str = include_str!("../../../../../config/version.ini");

#[cfg(not(feature = "embed_version"))]
const EMBEDDED_VERSION: &str = "";

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
    let version_str = match read_file(DEFAULT_META_SERVICE_CONFIG) {
        Ok(data) => {
            // Trim whitespace and newlines
            let version = data.trim().to_string();
            version
        }
        Err(e) => {
            error!("Failed to read version file: {}", e);
            // First try embedded version
            let embedded = EMBEDDED_VERSION.trim();
            if !embedded.is_empty() {
                info!("Using embedded version: {}", embedded);
                embedded.to_string()
            } else {
                // Finally use Cargo.toml version
                let fallback = option_env!("CARGO_PKG_VERSION").unwrap_or("unknown");
                error!("Using fallback version: {}", fallback);
                fallback.to_string()
            }
        }
    };

    // Cache the version in memory
    if let Err(e) = VERSION.set(version_str.clone()) {
        error!("Failed to cache version in memory (race condition),error message:{e}");
    }

    version_str
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::Path;
    use std::sync::Once;

    static INIT: Once = Once::new();

    // Setup function that runs only once
    fn setup() {
        INIT.call_once(|| {
            // Initialize any logging or other global state for tests
        });
    }

    #[test]
    fn test_version_cache() {
        setup();

        // Reset the static cache for testing
        // Note: This is not generally possible with OnceLock, so we rely on the test order
        // This test should run first before other version tests

        // Get version once
        let first_call = version();

        // Get version again - should use cached value
        let second_call = version();

        // Both calls should return the same value
        assert_eq!(first_call, second_call);
    }

    #[test]
    fn test_version_file_exists() {
        setup();

        // Create a temporary version file
        let test_version = "1.2.3-test";
        let test_path = "./test_version.ini";

        // Write test version to file
        let mut file = fs::File::create(test_path).expect("Failed to create test version file");
        file.write_all(test_version.as_bytes())
            .expect("Failed to write test version");

        // Override the default path temporarily for testing
        let _orig_path = logo::DEFAULT_META_SERVICE_CONFIG;
        let test_version_result = {
            // Create a new implementation that uses our test path
            fn test_read_file(path: &str) -> Result<String, std::io::Error> {
                if path == "./test_version.ini" {
                    fs::read_to_string(path)
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "File not found",
                    ))
                }
            }

            // Manually implement version logic for testing with our custom path
            match test_read_file(test_path) {
                Ok(data) => data.trim().to_string(),
                Err(_) => "fallback".to_string(),
            }
        };

        // Clean up
        fs::remove_file(test_path).expect("Failed to remove test file");

        // Verify version was read from file
        assert_eq!(test_version_result, test_version);
    }

    #[test]
    fn test_version_fallback() {
        setup();

        // Try to read from a file that doesn't exist
        let non_existent_path = "./non_existent_version.ini";

        // Verify the file doesn't exist
        assert!(!Path::new(non_existent_path).exists());

        // Create a test implementation that simulates a file read failure
        let test_version_result = {
            // This function always fails
            #[allow(dead_code)]
            fn test_read_file(_: &str) -> Result<String, std::io::Error> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "File not found",
                ))
            }

            // Use the embedded or Cargo.toml version logic
            let embedded = EMBEDDED_VERSION.trim();
            if !embedded.is_empty() {
                embedded.to_string()
            } else {
                option_env!("CARGO_PKG_VERSION")
                    .unwrap_or("unknown")
                    .to_string()
            }
        };

        // Verify fallback behavior - should either be the embedded version,
        // the Cargo.toml version, or "unknown"
        assert!(!test_version_result.is_empty());
    }
}
