use common_base::{config::broker_mqtt::init_broker_mqtt_conf_by_path, log::init_broker_mqtt_log};

pub fn setup() {
    let path = "/Users/bytedance/Desktop/code/robustmq-project/robustmq/config/mqtt-server.toml"
        .to_string();
    init_broker_mqtt_conf_by_path(&path);
    init_broker_mqtt_log();
}
