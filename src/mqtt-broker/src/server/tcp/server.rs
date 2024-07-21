use crate::{
    handler::{
        cache_manager::CacheManager,
        command::Command,
        connection::disconnect_connection,
        validator::{tcp_establish_connection_check, tcp_tls_establish_connection_check},
    },
    metrics::{metrics_request_queue, metrics_response_queue},
    server::{
        connection::NetworkConnection,
        connection_manager::ConnectionManager,
        packet::{RequestPackage, ResponsePackage},
        tcp::tls_server::read_tls_frame_process,
    },
};
use clients::poll::ClientPool;
use common_base::{
    config::broker_mqtt::broker_mqtt_conf,
    errors::RobustMQError,
    log::{debug, error, info},
};
use futures_util::StreamExt;
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

// U: codec: encoder + decoder
// S: message storage adapter
pub struct TcpServer<S> {
    command: Command<S>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
    client_poll: Arc<ClientPool>,
    accept_thread_num: usize,
    handler_process_num: usize,
    response_process_num: usize,
    stop_sx: broadcast::Sender<bool>,
}

impl<S> TcpServer<S>
where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    pub fn new(
        command: Command<S>,
        accept_thread_num: usize,
        handler_process_num: usize,
        response_process_num: usize,
        stop_sx: broadcast::Sender<bool>,
        connection_manager: Arc<ConnectionManager>,
        cache_manager: Arc<CacheManager>,
        client_poll: Arc<ClientPool>,
    ) -> Self {
        Self {
            command,
            cache_manager,
            client_poll,
            connection_manager,
            accept_thread_num,
            handler_process_num,
            response_process_num,
            stop_sx,
        }
    }

    pub async fn start(&self, port: u32) {
        let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)).await {
            Ok(tl) => tl,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };
        let (request_queue_sx, request_queue_rx) = mpsc::channel::<RequestPackage>(1000);
        let (response_queue_sx, response_queue_rx) = mpsc::channel::<ResponsePackage>(1000);

        let arc_listener = Arc::new(listener);
        self.acceptor_process(arc_listener.clone(), request_queue_sx)
            .await;
        self.handler_process(request_queue_rx, response_queue_sx)
            .await;
        self.response_process(response_queue_rx).await;
        info(format!(
            "MQTT TCP Server started successfully, listening port: {port}"
        ));
    }

    pub async fn start_tls(&self, port: u32) {
        let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)).await {
            Ok(tl) => tl,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };
        let (request_queue_sx, request_queue_rx) = mpsc::channel::<RequestPackage>(1000);
        let (response_queue_sx, response_queue_rx) = mpsc::channel::<ResponsePackage>(1000);

        let arc_listener = Arc::new(listener);
        self.acceptor_tls_process(arc_listener.clone(), request_queue_sx)
            .await;
        self.handler_process(request_queue_rx, response_queue_sx)
            .await;
        self.response_process(response_queue_rx).await;
        info(format!(
            "MQTT TCP TLS Server started successfully, listening port: {port}"
        ));
    }

    async fn acceptor_tls_process(
        &self,
        listener_arc: Arc<TcpListener>,
        request_queue_sx: Sender<RequestPackage>,
    ) {
        let conf = broker_mqtt_conf();

        let certs = match load_certs(&Path::new(&conf.mqtt.tls_cert)) {
            Ok(data) => data,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };

        let key = match load_key(&Path::new(&conf.mqtt.tls_key)) {
            Ok(data) => data,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };

        let config = match ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
        {
            Ok(data) => data,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };
        let tls_acceptor = TlsAcceptor::from(Arc::new(config));

        for index in 1..=self.accept_thread_num {
            let listener = listener_arc.clone();
            let connection_manager = self.connection_manager.clone();
            let mut stop_rx = self.stop_sx.subscribe();
            let raw_request_queue_sx = request_queue_sx.clone();
            let raw_tls_acceptor = tls_acceptor.clone();
            tokio::spawn(async move {
                debug(format!(
                    "TCP Server acceptor thread {} start successfully.",
                    index
                ));
                loop {
                    select! {
                        val = stop_rx.recv() =>{
                            match val{
                                Ok(flag) => {
                                    if flag {
                                        debug(format!("TCP Server acceptor thread {} stopped successfully.",index));
                                        break;
                                    }
                                }
                                Err(_) => {}
                            }
                        }
                        val = listener.accept()=>{
                            match val{
                                Ok((stream, addr)) => {
                                    info(format!("accept tcp tls connection:{:?}",addr));
                                    let stream = match raw_tls_acceptor.accept(stream).await{
                                        Ok(da) => da,
                                        Err(e) => {
                                            error(format!("Tls Accepter failed to read Stream with error message :{e:?}"));
                                            continue;
                                        }
                                    };
                                    let (r_stream, w_stream) = io::split(stream);
                                    let codec = MqttCodec::new(None);
                                    let read_frame_stream = FramedRead::new(r_stream, codec.clone());
                                    let mut  write_frame_stream = FramedWrite::new(w_stream, codec.clone());

                                    if !tcp_tls_establish_connection_check(&addr,&connection_manager,&mut write_frame_stream).await{
                                        continue;
                                    }

                                    let (connection_stop_sx, connection_stop_rx) = mpsc::channel::<bool>(1);
                                    let connection = NetworkConnection::new(
                                        crate::server::connection::NetworkConnectionType::TCPS,
                                        addr,
                                        Some(connection_stop_sx.clone())
                                    );
                                    connection_manager.add_connection(connection.clone());
                                    connection_manager.add_tcp_tls_write(connection.connection_id, write_frame_stream);

                                    read_tls_frame_process(read_frame_stream,connection,raw_request_queue_sx.clone(),connection_stop_rx);
                                }
                                Err(e) => {
                                    error(format!("TCP accept failed to create connection with error message :{:?}",e));
                                }
                            }
                        }
                    };
                }
            });
        }
    }

    async fn acceptor_process(
        &self,
        listener_arc: Arc<TcpListener>,
        request_queue_sx: Sender<RequestPackage>,
    ) {
        for index in 1..=self.accept_thread_num {
            let listener = listener_arc.clone();
            let connection_manager = self.connection_manager.clone();
            let mut stop_rx = self.stop_sx.subscribe();
            let raw_request_queue_sx = request_queue_sx.clone();

            tokio::spawn(async move {
                debug(format!(
                    "TCP Server acceptor thread {} start successfully.",
                    index
                ));
                loop {
                    select! {
                        val = stop_rx.recv() =>{
                            match val{
                                Ok(flag) => {
                                    if flag {
                                        debug(format!("TCP Server acceptor thread {} stopped successfully.",index));
                                        break;
                                    }
                                }
                                Err(_) => {}
                            }
                        }
                        val = listener.accept()=>{
                            match val{
                                Ok((stream, addr)) => {
                                    info(format!("accept tcp connection:{:?}",addr));

                                    let (r_stream, w_stream) = io::split(stream);
                                    let codec = MqttCodec::new(None);
                                    let read_frame_stream = FramedRead::new(r_stream, codec.clone());
                                    let mut  write_frame_stream = FramedWrite::new(w_stream, codec.clone());

                                    if !tcp_establish_connection_check(&addr,&connection_manager,&mut write_frame_stream).await{
                                        continue;
                                    }

                                    let (connection_stop_sx, connection_stop_rx) = mpsc::channel::<bool>(1);
                                    let connection = NetworkConnection::new(
                                        crate::server::connection::NetworkConnectionType::TCP,
                                        addr,
                                        Some(connection_stop_sx.clone())
                                    );
                                    connection_manager.add_connection(connection.clone());
                                    connection_manager.add_tcp_write(connection.connection_id, write_frame_stream);

                                    read_frame_process(read_frame_stream,connection,raw_request_queue_sx.clone(),connection_stop_rx);
                                }
                                Err(e) => {
                                    error(format!("TCP accept failed to create connection with error message :{:?}",e));
                                }
                            }
                        }
                    };
                }
            });
        }
    }

    async fn handler_process(
        &self,
        mut request_queue_rx: Receiver<RequestPackage>,
        response_queue_sx: Sender<ResponsePackage>,
    ) {
        let command = self.command.clone();
        let connect_manager = self.connection_manager.clone();
        let stop_sx = self.stop_sx.clone();
        let handler_process_num = self.handler_process_num.clone();

        tokio::spawn(async move {
            let mut child_process_list: HashMap<usize, Sender<RequestPackage>> = HashMap::new();
            handler_child_process(
                handler_process_num,
                stop_sx.clone(),
                connect_manager.clone(),
                command.clone(),
                &mut child_process_list,
                response_queue_sx.clone(),
            );

            let mut stop_rx = stop_sx.subscribe();
            let mut process_handler_seq = 1;
            loop {
                select! {
                    val = stop_rx.recv() =>{
                        match val{
                            Ok(flag) => {
                                if flag {
                                    debug("TCP Server handler thread stopped successfully.".to_string());
                                    break;
                                }
                            }
                            Err(_) => {}
                        }
                    },
                    val = request_queue_rx.recv()=>{
                        info(format!("7855666,{val:?}"));
                        if let Some(packet) = val{
                            // Try to deliver the request packet to the child handler until it is delivered successfully.
                            // Because some request queues may be full or abnormal, the request packets can be delivered to other child handlers.
                            loop{
                                let seq = if process_handler_seq > child_process_list.len(){
                                    1
                                } else {
                                    process_handler_seq
                                };

                                if let Some(handler_sx) = child_process_list.get(&seq){
                                    match handler_sx.try_send(packet.clone()){
                                        Ok(_) => {
                                            info(format!("success:{seq}"));
                                            break;
                                        }
                                        Err(err) => error(format!(
                                            "Failed to try write data to the handler process queue, error message: {:?}",
                                            err
                                        )),
                                    }
                                    process_handler_seq = process_handler_seq + 1;
                                }else{
                                    // In exceptional cases, if no available child handler can be found, the request packet is dropped.
                                    // If the client does not receive a return packet, it will retry the request.
                                    // Rely on repeated requests from the client to ensure that the request will eventually be processed successfully.
                                    error("No request packet processing thread available".to_string());
                                    break;
                                }
                            }

                        }
                    }
                }
            }
        });
    }

    async fn response_process(&self, mut response_queue_rx: Receiver<ResponsePackage>) {
        let connect_manager = self.connection_manager.clone();
        let mut stop_rx = self.stop_sx.subscribe();
        let response_process_num = self.response_process_num.clone();
        let cache_manager = self.cache_manager.clone();
        let client_poll = self.client_poll.clone();
        let stop_sx = self.stop_sx.clone();

        tokio::spawn(async move {
            let mut process_handler: HashMap<usize, Sender<ResponsePackage>> = HashMap::new();
            response_child_process(
                response_process_num,
                &mut process_handler,
                stop_sx,
                connect_manager,
                cache_manager,
                client_poll,
            );

            let mut process_handler_seq = 1;
            loop {
                select! {
                    val = stop_rx.recv() =>{
                        match val{
                            Ok(flag) => {
                                if flag {
                                    debug("TCP Server response process thread stopped successfully.".to_string());
                                    break;
                                }
                            }
                            Err(_) => {}
                        }
                    }

                    val = response_queue_rx.recv()=>{
                        if let Some(packet) = val{
                            metrics_request_queue("response-total", response_queue_rx.len() as i64);

                            loop{
                                let seq = if process_handler_seq > process_handler.len(){
                                    1
                                } else {
                                    process_handler_seq
                                };

                                if let Some(handler_sx) = process_handler.get(&seq){
                                    match handler_sx.try_send(packet.clone()){
                                        Ok(_) => {
                                            break;
                                        }
                                        Err(err) => error(format!(
                                            "Failed to write data to the response process queue, error message: {:?}",
                                            err
                                        )),
                                    }
                                    process_handler_seq = process_handler_seq + 1;
                                }else{
                                    error("No request packet processing thread available".to_string());
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}

fn read_frame_process(
    mut read_frame_stream: FramedRead<tokio::io::ReadHalf<tokio::net::TcpStream>, MqttCodec>,
    connection: NetworkConnection,
    request_queue_sx: Sender<RequestPackage>,
    mut connection_stop_rx: Receiver<bool>,
) {
    tokio::spawn(async move {
        loop {
            select! {
                val = connection_stop_rx.recv() =>{
                    if let Some(flag) = val{
                        if flag {
                            debug(format!("TCP connection 【{}】 acceptor thread stopped successfully.",connection.connection_id));
                            break;
                        }
                    }
                }
                val = read_frame_stream.next()=>{
                    if let Some(pkg) = val {
                        match pkg {
                            Ok(data) => {
                                let pack: MQTTPacket = data.try_into().unwrap();
                                info(format!("revc tcp packet:{:?}", pack));
                                let package =
                                    RequestPackage::new(connection.connection_id, connection.addr, pack);
                                match request_queue_sx.send(package).await {
                                    Ok(_) => {}
                                    Err(err) => error(format!("Failed to write data to the request queue, error message: {:?}",err)),
                                }
                            }
                            Err(e) => {
                                debug(format!("TCP connection parsing packet format error message :{:?}",e))
                            }
                        }
                    }
                }
            }
        }
    });
}

fn handler_child_process<S>(
    handler_process_num: usize,
    stop_sx: broadcast::Sender<bool>,
    connection_manager: Arc<ConnectionManager>,
    command: Command<S>,
    child_process_list: &mut HashMap<usize, Sender<RequestPackage>>,
    response_queue_sx: Sender<ResponsePackage>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    for index in 1..=handler_process_num {
        let (child_hendler_sx, mut child_process_rx) = mpsc::channel::<RequestPackage>(1000);
        child_process_list.insert(index, child_hendler_sx.clone());

        let mut raw_stop_rx = stop_sx.subscribe();
        let raw_connect_manager = connection_manager.clone();
        let raw_response_queue_sx = response_queue_sx.clone();
        let mut raw_command = command.clone();

        tokio::spawn(async move {
            debug(format!(
                "TCP Server handler process thread {} start successfully.",
                index
            ));
            loop {
                select! {
                    val = raw_stop_rx.recv() =>{
                        match val{
                            Ok(flag) => {
                                if flag {
                                    debug(format!("TCP Server handler process thread {} stopped successfully.",index));
                                    break;
                                }
                            }
                            Err(_) => {}
                        }
                    },
                    val = child_process_rx.recv()=>{
                        if let Some(packet) = val{
                            info(format!("1:{:?}", packet));
                            if let Some(connect) = raw_connect_manager.get_connect(packet.connection_id) {
                                info(format!("2:{:?}", packet));
                                if let Some(resp) = raw_command
                                    .apply(raw_connect_manager.clone(), connect, packet.addr, packet.packet)
                                    .await
                                {
                                    info(format!("3:"));
                                    let response_package = ResponsePackage::new(packet.connection_id, resp);
                                    match raw_response_queue_sx.send(response_package).await {
                                        Ok(_) => {}
                                        Err(err) => error(format!(
                                            "Failed to write data to the response queue, error message: {:?}",
                                            err
                                        )),
                                    }
                                } else {
                                    info("No backpacking is required for this request".to_string());
                                }
                            } else {
                                error(RobustMQError::NotFoundConnectionInCache(packet.connection_id).to_string());
                            }
                        }
                    }
                }
            }
        });
    }
}

fn response_child_process(
    response_process_num: usize,
    process_handler: &mut HashMap<usize, Sender<ResponsePackage>>,
    stop_sx: broadcast::Sender<bool>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
    client_poll: Arc<ClientPool>,
) {
    for index in 1..=response_process_num {
        let (response_process_sx, mut response_process_rx) = mpsc::channel::<ResponsePackage>(100);
        process_handler.insert(index, response_process_sx.clone());

        let mut raw_stop_rx = stop_sx.subscribe();
        let raw_connect_manager = connection_manager.clone();
        let raw_cache_manager = cache_manager.clone();
        let raw_client_poll = client_poll.clone();
        tokio::spawn(async move {
            debug(format!(
                "TCP Server response process thread {index} start successfully."
            ));

            loop {
                select! {
                    val = raw_stop_rx.recv() =>{
                        match val{
                            Ok(flag) => {
                                if flag {
                                    debug(format!("TCP Server response process thread {index} stopped successfully."));
                                    break;
                                }
                            }
                            Err(_) => {}
                        }
                    },
                    val = response_process_rx.recv()=>{
                        if let Some(response_package) = val{
                            let lable = format!("handler-{}",index);
                            metrics_response_queue(&lable, response_process_rx.len() as i64);

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
                                            error(e.to_string());
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
                                    ).await{
                                        Ok(()) => {},
                                        Err(e) => error(e.to_string())
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
