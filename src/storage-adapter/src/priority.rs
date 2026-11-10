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

use metadata_struct::mq9::Priority;

pub fn storage_priority_tag(prefix: &str, priority: &Priority) -> String {
    format!("{}_{}", prefix, priority)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_priority_tag_uses_priority_wire_name() {
        assert_eq!(
            storage_priority_tag("tenant_user", &Priority::Critical),
            "tenant_user_critical"
        );
        assert_eq!(
            storage_priority_tag("tenant_user", &Priority::Urgent),
            "tenant_user_urgent"
        );
        assert_eq!(
            storage_priority_tag("tenant_user", &Priority::Normal),
            "tenant_user_normal"
        );
    }
}
