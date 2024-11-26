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

use std::sync::Arc;

use common_base::tools::now_mills;
use log::{error, info};
use protocol::mqtt::codec::parse_mqtt_packet_to_name;
use serde::{Deserialize, Serialize};

use crate::handler::cache::CacheManager;
use crate::server::packet::RequestPackage;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct SlowRequestMs {
    command: String,
    time_ms: u128,
}

// total processing time of the request packet was recorded
/// 该方法主要用来记录消息处理以及传输整个流程所消耗的时间，如果开启了慢订阅统计功能
/// 此时我们会将超过阈值的*订阅者*和*主题*插入或更新到*慢订阅列表*中，并且会按照时延的
/// 高低对列表进行排序
/// 对于慢订阅列表：
/// - 列表中数据按照时延从大到小排名，最多可以记录前1000条数据
///     - 后期可以优化成自定义参数 - 优化成自定义参数可能就会产生排序变慢的影响，但是后期可以尝试使用跳表结构？
/// - 列表记录的是*订阅者-主题*数据，并不是每一条超过阈值的消息
///     - 使用的数据结构需要有*订阅者-主题*以及对应的*发生时间* 元组结构 (订阅者，主题，发生时间)
///       SlowMessage部分是否就是慢消息结构体？
/// - 记录产生时，如果列表中不存在相同的记录则插入到列表中， 否则将更新其发生时间并对整个列表重新排名
///   使用数组，查询时间O(n), 排序时间O(n)， 默认设置为1000长度的数组->后期可以优化成按需扩容
///   按道理这个插入的结构应该本身就是有序的，先发生的在数组前面，后发生的在数组后面
///   每次需要查一遍数组来判定是否存在，优化->位图，判定不存在，不存在的话就不查了，否则查询对应的位置
///   查到位置后利用切片把后面的前移动一位，在数组最后插入就可以保证有序了
/// - 记录产生后，有效时长内（默认300秒）没有再次触发将被移除统计列表
///     - 每5分钟移除一批数据，连续还是链式 -> 连续，切片
/// 消息完成传输的定义：- 利用模式匹配拿到对应的内容即可
/// - QoS0: 消息成功发出
/// - QoS1: MQTT Server收到客户端的PUBACK包
/// - Qos2: MQTT Server收到客户端的PUBCOMP包
pub fn try_record_total_request_ms(cache_manager: Arc<CacheManager>, package: RequestPackage) {
    let cluster_config = cache_manager.get_cluster_info();
    if !cluster_config.slow.enable {
        return;
    }

    let whole = cluster_config.slow.whole_ms;
    let time_ms = now_mills() - package.receive_ms;
    if time_ms < whole {
        return;
    }

    let command = parse_mqtt_packet_to_name(package.packet);
    let slow = SlowRequestMs { command, time_ms };

    match serde_json::to_string(&slow) {
        Ok(data) => info!("{}", data),
        Err(e) => error!(
            "Failed to serialize slow subscribe message with error message :{}",
            e.to_string()
        ),
    }
}
