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

use tracing::info;

pub const DEFAULT_PLACEMENT_CENTER_CONFIG: &str = "./config/version.ini";

pub fn banner() {
    const B: &str = r"

     /$$$$$$$            /$$                             /$$     /$$      /$$  /$$$$$$
    | $$__  $$          | $$                            | $$    | $$$    /$$$ /$$__  $$
    | $$  \ $$  /$$$$$$ | $$$$$$$  /$$   /$$  /$$$$$$$ /$$$$$$  | $$$$  /$$$$| $$  \ $$
    | $$$$$$$/ /$$__  $$| $$__  $$| $$  | $$ /$$_____/|_  $$_/  | $$ $$/$$ $$| $$  | $$
    | $$__  $$| $$  \ $$| $$  \ $$| $$  | $$|  $$$$$$   | $$    | $$  $$$| $$| $$  | $$
    | $$  \ $$| $$  | $$| $$  | $$| $$  | $$ \____  $$  | $$ /$$| $$\  $ | $$| $$/$$ $$
    | $$  | $$|  $$$$$$/| $$$$$$$/|  $$$$$$/ /$$$$$$$/  |  $$$$/| $$ \/  | $$|  $$$$$$/
    |__/  |__/ \______/ |_______/  \______/ |_______/    \___/  |__/     |__/ \____ $$$
                                                                                   \__/

    ";

    info!("{B}");

    // Print version information
    let version = super::version();
    info!("Version: {}", version);

    // Print components information
    info!("Components:");
    info!("  - MQTT Broker");
    info!("  - Placement Center");
}

#[cfg(test)]
mod tests {

    use super::banner;

    #[test]
    fn banner_test() {
        banner();
    }
}
