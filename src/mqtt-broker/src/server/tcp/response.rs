use crate::{
    handler::{
        cache::CacheManager,
        command::Command,
        connection::disconnect_connection,
        validator::{tcp_establish_connection_check, tcp_tls_establish_connection_check},
    },
    observability::{
        metrics::{
            packets::{record_received_error_metrics, record_received_metrics},
            server::{metrics_request_queue, metrics_response_queue},
        },
        slow::request::try_record_total_request_ms,
    },
    server::{
        connection::{NetworkConnection, NetworkConnectionType},
        connection_manager::ConnectionManager,
        packet::{RequestPackage, ResponsePackage},
        tcp::{
            handler::handler_process,
            tls_server::{acceptor_tls_process, read_tls_frame_process},
        },
    },
    subscribe::subscribe_manager::SubscribeManager,
};
use clients::poll::ClientPool;
use common_base::{config::broker_mqtt::broker_mqtt_conf, error::mqtt_broker::MQTTBrokerError};
use futures_util::StreamExt;
use log::{debug, error, info};
use protocol::mqtt::{
    codec::{MQTTPacketWrapper, MqttCodec},
    common::MQTTPacket,
};
use std::{collections::HashMap, path::Path, sync::Arc};
use storage_adapter::storage::StorageAdapter;
use tokio::{
    io, select,
    sync::mpsc::{self, Receiver, Sender},
};
use tokio::{net::TcpListener, sync::broadcast};
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};
use tokio_util::codec::{FramedRead, FramedWrite};

use super::tls_server::{load_certs, load_key};

pub(crate) fn response_child_process(
    response_process_num: usize,
    process_handler: &mut HashMap<usize, Sender<ResponsePackage>>,
    stop_sx: broadcast::Sender<bool>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    client_poll: Arc<ClientPool>,
) {
    for index in 1..=response_process_num {
        let (response_process_sx, mut response_process_rx) = mpsc::channel::<ResponsePackage>(100);
        process_handler.insert(index, response_process_sx.clone());

        let mut raw_stop_rx = stop_sx.subscribe();
        let raw_connect_manager = connection_manager.clone();
        let raw_cache_manager = cache_manager.clone();
        let raw_client_poll = client_poll.clone();
        let raw_subscribe_manager = subscribe_manager.clone();
        tokio::spawn(async move {
            debug!("TCP Server response process thread {index} start successfully.");

            loop {
                select! {
                    val = raw_stop_rx.recv() =>{
                        match val{
                            Ok(flag) => {
                                if flag {
                                    debug!("TCP Server response process thread {index} stopped successfully.");
                                    break;
                                }
                            }
                            Err(_) => {}
                        }
                    },
                    val = response_process_rx.recv()=>{
                        if let Some(response_package) = val{
                            let lable = format!("handler-{}",index);
                            metrics_response_queue(&lable, response_process_rx.len());

                            if let Some(protocol) =
                            raw_connect_manager.get_connect_protocol(response_package.connection_id)
                            {
                                let packet_wrapper = MQTTPacketWrapper {
                                    protocol_version: protocol.into(),
                                    packet: response_package.packet.clone(),
                                };
                                match raw_connect_manager
                                    .write_tcp_frame(response_package.connection_id, packet_wrapper)
                                    .await{
                                        Ok(()) => {},
                                        Err(e) => {
                                            error!("{}",e);
                                            raw_connect_manager.clonse_connect(response_package.connection_id).await;
                                            break;
                                        }
                                    }
                            }

                            if let MQTTPacket::Disconnect(_, _) = response_package.packet {
                                if let Some(connection) = raw_cache_manager.get_connection(response_package.connection_id){
                                    match disconnect_connection(
                                        &connection.client_id,
                                        connection.connect_id,
                                        &raw_cache_manager,
                                        &raw_client_poll,
                                        &raw_connect_manager,
                                        &raw_subscribe_manager,
                                    ).await{
                                        Ok(()) => {},
                                        Err(e) => error!("{}",e)
                                    };
                                }
                                break;
                            }
                        }
                    }
                }
            }
        });
    }
}
