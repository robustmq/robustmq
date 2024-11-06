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

pub fn data_fold_shard(namespace: &str, shard_name: &str, data_fold: &str) -> String {
    let file_name = format!("{}/{}", namespace, shard_name);
    format!("{}/{}", data_fold, file_name)
}

pub fn data_file_segment(data_fold: &str, segment_no: u32) -> String {
    format!("{}/{}.msg", data_fold, segment_no)
}
