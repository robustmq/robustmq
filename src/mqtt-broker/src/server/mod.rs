use common_base::{
    config::broker_mqtt::{broker_mqtt_conf, BrokerMQTTConfig},
    log::info,
};
use protocol::{mqttv4::codec::Mqtt4Codec, mqttv5::codec::Mqtt5Codec};

use self::tcp::tcp_server::TcpServer;

pub mod grpc;
pub mod quic;
pub mod tcp;
pub mod websocket;

pub async fn start_server() {
    let conf = broker_mqtt_conf();

    if conf.mqtt.mqtt4_enable {
        start_mqtt4_server(conf).await;
    }

    if conf.mqtt.mqtt5_enable {
        start_mqtt5_server(conf).await;
    }
}

async fn start_mqtt4_server(conf: &BrokerMQTTConfig) {
    let port = conf.mqtt.mqtt4_port;
    let codec = Mqtt4Codec::new();
    let server = TcpServer::<Mqtt4Codec>::new(
        conf.network_tcp.accept_thread_num,
        conf.network_tcp.max_connection_num,
        conf.network_tcp.request_queue_size,
        conf.network_tcp.handler_thread_num,
        conf.network_tcp.response_queue_size,
        conf.network_tcp.response_thread_num,
        conf.network_tcp.lock_max_try_mut_times,
        conf.network_tcp.lock_try_mut_sleep_time_ms,
        codec,
    );
    server.start(port).await;
    info(format!(
        "MQTT4 TCP Server started successfully, listening port: {}",
        port
    ));
}

async fn start_mqtt5_server(conf: &BrokerMQTTConfig) {
    let codec = Mqtt5Codec::new();
    let port = conf.mqtt.mqtt5_port;
    let server = TcpServer::<Mqtt5Codec>::new(
        conf.network_tcp.accept_thread_num,
        conf.network_tcp.max_connection_num,
        conf.network_tcp.request_queue_size,
        conf.network_tcp.handler_thread_num,
        conf.network_tcp.response_queue_size,
        conf.network_tcp.response_thread_num,
        conf.network_tcp.lock_max_try_mut_times,
        conf.network_tcp.lock_try_mut_sleep_time_ms,
        codec,
    );
    server.start(port).await;
    info(format!(
        "MQTT5 TCP Server started successfully, listening port: {}",
        port
    ));
}

async fn start_quic_server() {}
