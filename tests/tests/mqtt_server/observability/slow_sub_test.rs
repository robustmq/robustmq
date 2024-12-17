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
    use std::path::PathBuf;

    use mqtt_broker::observability::slow::sub::read_slow_sub_record;
    use protocol::broker_mqtt::broker_mqtt_admin::ListSlowSubscribeRequest;

    #[test]
    fn test_read_slow_record_path_is_empty() {
        let path = "";
        let path_buf = PathBuf::new().join(path);
        let record = ListSlowSubscribeRequest {
            sub_name: "/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e".to_string(),
            client_id: "".to_string(),
            topic: "".to_string(),
            list: 5,
            sort: "asc".to_string(),
        };
        let result = read_slow_sub_record(record, path_buf).unwrap();
        assert_eq!(0, result.clone().len())
    }

    #[test]
    fn test_read_slow_record_param_is_one_1() {
        let path = "./tests/mqtt_server/example/slow_sub.log";
        let path_buf = PathBuf::new().join(path);
        let record = ListSlowSubscribeRequest {
            sub_name: "/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e".to_string(),
            client_id: "".to_string(),
            topic: "".to_string(),
            list: 5,
            sort: "asc".to_string(),
        };

        let result = read_slow_sub_record(record, path_buf).unwrap();

        assert_eq!(5, result.clone().len());
        assert_eq!(
            result.clone()[0],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","time_ms":13,"node_info":"RobustMQ-MQTT@182.22.194.185","create_time":1733488597}"#
        );
        assert_eq!(
            result.clone()[1],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","time_ms":3,"node_info":"RobustMQ-MQTT@172.22.111.185","create_time":1733398597}"#
        );
        assert_eq!(
            result.clone()[2],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/request/131edb8526804e80b32b387fa2340d35","time_ms":143,"node_info":"RobustMQ-MQTT@172.22.111.85","create_time":1733864597}"#
        );
        assert_eq!(
            result.clone()[3],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"","topic":"","time_ms":543,"node_info":"RobustMQ-MQTT@172.22.194.185","create_time":1733898597}"#
        );
        assert_eq!(
            result.clone()[4],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/request/131edb8526804e80b32b387fa2340d35","time_ms":543,"node_info":"RobustMQ-MQTT@192.32.194.185","create_time":1733858597}"#
        );
    }

    #[test]
    fn test_read_slow_record_param_is_one_2() {
        let path = "./tests/mqtt_server/example/slow_sub.log";
        let path_buf = PathBuf::new().join(path);
        let record = ListSlowSubscribeRequest {
            sub_name: "".to_string(),
            client_id: "ed280344fec44aad8a78b00ff1dec99a".to_string(),
            topic: "".to_string(),
            list: 5,
            sort: "asc".to_string(),
        };

        let result = read_slow_sub_record(record, path_buf).unwrap();

        assert_eq!(5, result.clone().len());
        assert_eq!(
            result.clone()[0],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/request/131edb8526804e80b32b387fa2340d35","time_ms":543,"node_info":"RobustMQ-MQTT@172.22.194.185","create_time":1733898597}"#
        );
        assert_eq!(
            result.clone()[1],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","time_ms":13,"node_info":"RobustMQ-MQTT@182.22.194.185","create_time":1733488597}"#
        );
        assert_eq!(
            result.clone()[2],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","time_ms":3,"node_info":"RobustMQ-MQTT@172.22.111.185","create_time":1733398597}"#
        );
        assert_eq!(
            result.clone()[3],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/request/131edb8526804e80b32b387fa2340d35","time_ms":143,"node_info":"RobustMQ-MQTT@172.22.111.85","create_time":1733864597}"#
        );
        assert_eq!(
            result.clone()[4],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/request/131edb8526804e80b32b387fa2340d35","time_ms":543,"node_info":"RobustMQ-MQTT@192.32.194.185","create_time":1733858597}"#
        );
    }

    #[test]
    fn test_read_slow_record_param_is_one_3() {
        let path = "./tests/mqtt_server/example/slow_sub.log";
        let path_buf = PathBuf::new().join(path);
        let record = ListSlowSubscribeRequest {
            sub_name: "".to_string(),
            client_id: "".to_string(),
            topic: "/request/131edb8526804e80b32b387fa2340d35".to_string(),
            list: 5,
            sort: "asc".to_string(),
        };

        let result = read_slow_sub_record(record, path_buf).unwrap();

        assert_eq!(4, result.clone().len());
        assert_eq!(
            result.clone()[0],
            r#"{"sub_name":"/request/131edb8526804e80b32b387fa2340d35","client_id":"49e10a8d8a494cefa904a00dcf0b30af","topic":"/request/131edb8526804e80b32b387fa2340d35","time_ms":273,"node_info":"RobustMQ-MQTT@172.22.194.185","create_time":1733898601}"#
        );
        assert_eq!(
            result.clone()[1],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/request/131edb8526804e80b32b387fa2340d35","time_ms":543,"node_info":"RobustMQ-MQTT@172.22.194.185","create_time":1733898597}"#
        );
        assert_eq!(
            result.clone()[2],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/request/131edb8526804e80b32b387fa2340d35","time_ms":143,"node_info":"RobustMQ-MQTT@172.22.111.85","create_time":1733864597}"#
        );
        assert_eq!(
            result.clone()[3],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/request/131edb8526804e80b32b387fa2340d35","time_ms":543,"node_info":"RobustMQ-MQTT@192.32.194.185","create_time":1733858597}"#
        );
    }

    #[test]
    fn test_read_slow_record_param_is_two() {
        let path = "./tests/mqtt_server/example/slow_sub.log";
        let path_buf = PathBuf::new().join(path);
        let record = ListSlowSubscribeRequest {
            sub_name: "/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e".to_string(),
            client_id: "ed280344fec44aad8a78b00ff1dec99a".to_string(),
            topic: "".to_string(),
            list: 5,
            sort: "asc".to_string(),
        };

        let result = read_slow_sub_record(record, path_buf).unwrap();

        assert_eq!(5, result.clone().len());
        assert_eq!(
            result.clone()[0],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/request/131edb8526804e80b32b387fa2340d35","time_ms":543,"node_info":"RobustMQ-MQTT@172.22.194.185","create_time":1733898597}"#
        );
        assert_eq!(
            result.clone()[1],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","time_ms":13,"node_info":"RobustMQ-MQTT@182.22.194.185","create_time":1733488597}"#
        );
        assert_eq!(
            result.clone()[2],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","time_ms":3,"node_info":"RobustMQ-MQTT@172.22.111.185","create_time":1733398597}"#
        );
        assert_eq!(
            result.clone()[3],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/request/131edb8526804e80b32b387fa2340d35","time_ms":143,"node_info":"RobustMQ-MQTT@172.22.111.85","create_time":1733864597}"#
        );
        assert_eq!(
            result.clone()[4],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/request/131edb8526804e80b32b387fa2340d35","time_ms":543,"node_info":"RobustMQ-MQTT@192.32.194.185","create_time":1733858597}"#
        );
    }

    #[test]
    fn test_read_slow_record_param_is_three() {
        let path = "./tests/mqtt_server/example/slow_sub.log";
        let path_buf = PathBuf::new().join(path);
        let record = ListSlowSubscribeRequest {
            sub_name: "/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e".to_string(),
            client_id: "ed280344fec44aad8a78b00ff1dec99a".to_string(),
            topic: "/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e".to_string(),
            list: 5,
            sort: "asc".to_string(),
        };

        let result = read_slow_sub_record(record, path_buf).unwrap();

        assert_eq!(3, result.clone().len());
        assert_eq!(
            result.clone()[0],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","time_ms":543,"node_info":"RobustMQ-MQTT@172.22.194.185","create_time":1733898597}"#
        );
        assert_eq!(
            result.clone()[1],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","time_ms":13,"node_info":"RobustMQ-MQTT@182.22.194.185","create_time":1733488597}"#
        );
        assert_eq!(
            result.clone()[2],
            r#"{"sub_name":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","client_id":"ed280344fec44aad8a78b00ff1dec99a","topic":"/packet_tcp_ssl/7fce56aa49ef4cea90dc4be77d6a775e","time_ms":3,"node_info":"RobustMQ-MQTT@172.22.111.185","create_time":1733398597}"#
        );
    }
}
