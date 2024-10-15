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

pub mod placement_center_journal {
    tonic::include_proto!("placement.center.journal");
}

pub mod placement_center_inner {
    tonic::include_proto!("placement.center.inner");
}

pub mod placement_center_kv {
    tonic::include_proto!("placement.center.kv");
}

pub mod placement_center_mqtt {
    tonic::include_proto!("placement.center.mqtt");
}

pub mod placement_center_openraft {
    tonic::include_proto!("placement.center.openraft");
}

