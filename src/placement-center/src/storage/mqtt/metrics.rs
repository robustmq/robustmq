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

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//  http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};

#[derive(Eq, Hash, Clone, Debug, PartialEq, Default, EncodeLabelSet)]
pub(crate) struct TopicLabel {
    pub(crate) cluster_name: String,
    pub(crate) topic_type: TopicType,
}

#[derive(Eq, Hash, Clone, Debug, PartialEq, Default, EncodeLabelValue)]
pub(crate) enum TopicType {
    System,

    #[default]
    Normal,
}

common_base::register_gauge_metric!(TOPIC_NUM, "topic_num", "Total number of topic", TopicLabel);

pub fn metrics_topic_num_inc(cluster_name: &str, topic_type: TopicType) {
    let label = TopicLabel {
        cluster_name: cluster_name.to_string(),
        topic_type,
    };

    common_base::gauge_metric_inc!(TOPIC_NUM, label)
}

pub fn metrics_topic_num_desc(cluster_name: &str, topic_type: TopicType) {
    let label = TopicLabel {
        cluster_name: cluster_name.to_string(),
        topic_type,
    };

    common_base::gauge_metric_inc_by!(TOPIC_NUM, label, -1)
}
