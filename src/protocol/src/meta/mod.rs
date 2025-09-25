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

#![cfg_attr(any(), rustfmt::skip)]
#![allow(clippy::all)]

pub mod meta_service_journal {
    tonic::include_proto!("meta.service.journal");
}

pub mod meta_service_inner {
    tonic::include_proto!("meta.service.inner");
}

pub mod meta_service_kv {
    tonic::include_proto!("meta.service.kv");
}

pub mod meta_service_mqtt {
    tonic::include_proto!("meta.service.mqtt");
}

pub mod meta_service_openraft {
    tonic::include_proto!("meta.service.openraft");
}
