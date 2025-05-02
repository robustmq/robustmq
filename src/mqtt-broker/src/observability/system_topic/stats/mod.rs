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

// Stats
// connections
pub const SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_COUNT: &str =
    "$SYS/brokers/${node}/stats/connections/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_CONNECTIONS_MAX: &str =
    "$SYS/brokers/${node}/stats/connections/max";
// subscribe
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_COUNT: &str =
    "$SYS/brokers/${node}/stats/suboptions/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBOPTIONS_MAX: &str =
    "$SYS/brokers/${node}/stats/suboptions/max";
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIBERS_COUNT: &str =
    "$SYS/brokers/${node}/stats/subscribers/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_COUNT: &str =
    "$SYS/brokers/${node}/stats/subscriptions/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_COUNT: &str =
    "$SYS/brokers/${node}/stats/subscriptions/shared/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_SUBSCRIPTIONS_SHARED_MAX: &str =
    "$SYS/brokers/${node}/stats/subscriptions/shared/max";
// topics
pub const SYSTEM_TOPIC_BROKERS_STATS_TOPICS_COUNT: &str = "$SYS/brokers/${node}/stats/topics/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_TOPICS_MAX: &str = "$SYS/brokers/${node}/stats/topics/max";
// routes
pub const SYSTEM_TOPIC_BROKERS_STATS_ROUTES_COUNT: &str = "$SYS/brokers/${node}/stats/routes/count";
pub const SYSTEM_TOPIC_BROKERS_STATS_ROUTES_MAX: &str = "$SYS/brokers/${node}/stats/routes/max";
