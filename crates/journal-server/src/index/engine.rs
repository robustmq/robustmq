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

use crate::core::consts::DB_COLUMN_FAMILY_INDEX;

pub fn column_family_list() -> Vec<String> {
    vec![DB_COLUMN_FAMILY_INDEX.to_string()]
}

pub fn storage_data_fold(data_fold: &Vec<String>) -> String {
    if let Some(fold) = data_fold.first() {
        return format!("{fold}_index");
    }
    panic!("No configuration data storage directory, configuration info :{data_fold:?}");
}
