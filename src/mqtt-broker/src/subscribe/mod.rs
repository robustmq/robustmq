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

use protocol::mqtt::common::{Publish, PublishProperties};
use subscriber::Subscriber;

pub mod sub_common;
pub mod sub_exclusive;
pub mod sub_share_follower;
pub mod sub_share_leader;
pub mod subscribe_manager;
pub mod subscriber;

#[derive(Clone, Default, Debug)]
pub(crate) struct SubPublishParam {
    pub subscribe: Subscriber,
    pub publish: Publish,
    pub properties: Option<PublishProperties>,
    pub create_time: u128,
    pub pkid: u16,
    pub group_id: String,
}

impl SubPublishParam {
    pub fn new(
        subscribe: Subscriber,
        publish: Publish,
        properties: Option<PublishProperties>,
        create_time: u128,
        group_id: String,
        pkid: u16,
    ) -> Self {
        SubPublishParam {
            subscribe,
            publish,
            properties,
            create_time,
            pkid,
            group_id,
        }
    }
}
