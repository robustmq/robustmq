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

pub struct Segment {
    pub namespace: String,
    pub shard_name: String,
    pub segment_no: u64,
    pub start_segment_no: u64,
    pub last_segment_no: u64,
    pub file: String,
}

impl Segment {
    pub fn new(namespace: String, shard_name: String, segment_no: u64, data_fold: String) -> Self {
        let filet_path = build_file_path(&namespace, &shard_name, &data_fold, segment_no);
        Segment {
            namespace,
            shard_name,
            segment_no,
            start_segment_no: 0,
            last_segment_no: 0,
            file: filet_path,
        }
    }

    pub fn open(&self) {}

    pub fn write(&self) {}

    pub fn read(&self) {}
}

fn build_file_path(namespace: &str, shard_name: &str, data_fold: &str, segment_no: u64) -> String {
    let file_name = format!("{}/{}-{}", namespace, shard_name, segment_no);
    format!("{}/{}", data_fold, file_name)
}

#[cfg(test)]
mod tests {
    #[test]
    fn segment_test() {}
}
