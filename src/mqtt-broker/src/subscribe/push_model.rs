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

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum PushModel {
    QuickFailure,
    RetryFailure,
}

pub fn get_push_model(client_id: &str, topic_name: &str) -> PushModel {
    if let Some(model) = get_push_model_by_client_id(client_id) {
        return model;
    }

    if let Some(model) = get_push_model_by_topic_name(topic_name) {
        return model;
    }
    PushModel::QuickFailure
}

fn get_push_model_by_topic_name(_topic_name: &str) -> Option<PushModel> {
    Some(PushModel::QuickFailure)
}

fn get_push_model_by_client_id(_client_id: &str) -> Option<PushModel> {
    None
}
