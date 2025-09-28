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

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TestConfig {
    pub roles: Vec<String>,
}

fn main() {
    let config = TestConfig {
        roles: vec!["meta".to_string(), "broker".to_string()],
    };
    
    let json = serde_json::to_string_pretty(&config).unwrap();
    println!("JSON output:");
    println!("{}", json);
    
    // Test direct array serialization
    let roles = vec!["meta".to_string(), "broker".to_string()];
    let roles_json = serde_json::to_string_pretty(&roles).unwrap();
    println!("\nDirect array output:");
    println!("{}", roles_json);
}
