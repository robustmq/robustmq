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

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn create_rust_pb() {
        tonic_build::configure()
            .build_server(true)
            .out_dir("src/broker_server/generate") // you can change the generated code's location
            .compile(
                &[
                    "src/broker_server/proto/admin.proto",
                    "src/broker_server/proto/placement.proto",
                    ],
                &["src/broker_server/proto"], // specify the root location to search proto dependencies
            )
            .unwrap();

        tonic_build::configure()
            .build_server(true)
            .out_dir("src/placement_center/generate") // you can change the generated code's location
            .compile(
                &[
                    "src/placement_center/proto/common.proto",
                    "src/placement_center/proto/journal.proto",
                    "src/placement_center/proto/kv.proto",
                    "src/placement_center/proto/mqtt.proto",
                    "src/placement_center/proto/placement.proto",
                    "src/placement_center/proto/openraft.proto",
                    ],
                &["src/placement_center/proto"], // specify the root location to search proto dependencies
            )
            .unwrap();

        tonic_build::configure()
            .build_server(false)
            .out_dir("src/journal_server/generate/protocol") // you can change the generated code's location
            .compile(
                &[
                    "src/journal_server/proto/protocol/header.proto",
                    "src/journal_server/proto/protocol/fetch.proto",
                    "src/journal_server/proto/protocol/produce.proto",
                ],
                &["src/journal_server/proto/protocol/"], // specify the root location to search proto dependencies
            )
            .unwrap();

        tonic_build::configure()
            .build_server(false)
            .out_dir("src/journal_server/generate/record") // you can change the generated code's location
            .compile(
                &[
                    "src/journal_server/proto/record/record.proto",
                ],
                &["src/journal_server/proto/record"], // specify the root location to search proto dependencies
            )
            .unwrap();
    }
}
