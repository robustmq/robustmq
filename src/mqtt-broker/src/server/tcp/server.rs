use crate::{
    handler::{command::Command, validator::establish_connection_check},
    metrics::{metrics_request_queue, metrics_response_queue},
    server::{
        connection::NetworkConnection,
        connection_manager::ConnectionManager,
        packet::{RequestPackage, ResponsePackage},
    },
};
use common_base::{
    errors::RobustMQError,
    log::{debug, error, info},
};
use futures_util::StreamExt;
use protocol::mqtt::{
    codec::{MQTTPacketWrapper, MqttCodec},
    common::MQTTPacket,
};
use std::{collections::HashMap, sync::Arc};
use storage_adapter::storage::StorageAdapter;
use tokio::{
    io, select,
    sync::broadcast::{Receiver, Sender},
};
use tokio::{net::TcpListener, sync::broadcast};
use tokio_util::codec::{FramedRead, FramedWrite};

// U: codec: encoder + decoder
// S: message storage adapter
pub struct TcpServer<S> {
    command: Command<S>,
    connection_manager: Arc<ConnectionManager>,
    accept_thread_num: usize,
    handler_process_num: usize,
    response_process_num: usize,
    request_queue_sx: Sender<RequestPackage>,
    response_queue_sx: Sender<ResponsePackage>,
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
        request_queue_sx: Sender<RequestPackage>,
        response_queue_sx: Sender<ResponsePackage>,
        stop_sx: broadcast::Sender<bool>,
        connection_manager: Arc<ConnectionManager>,
    ) -> Self {
        Self {
            command,
            connection_manager,
            accept_thread_num,
            handler_process_num,
            response_process_num,
            request_queue_sx,
            response_queue_sx,
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

        let arc_listener = Arc::new(listener);
        self.acceptor_process(arc_listener.clone()).await;
        self.handler_process().await;
        self.response_process().await;
        self.wait_stop_server().await;
    }

    pub async fn start_tls(&self, port: u32) {
        let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)).await {
            Ok(tl) => tl,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };
        let listener = Arc::new(listener);
        self.acceptor_process(listener.clone()).await;
        self.handler_process().await;
        self.response_process().await;
        self.wait_stop_server().await;
    }

    async fn acceptor_process(&self, listener_arc: Arc<TcpListener>) {
        for index in 1..=self.accept_thread_num {
            let listener = listener_arc.clone();
            let request_queue_sx = self.request_queue_sx.clone();
            let connection_manager = self.connection_manager.clone();
            let mut stop_rx = self.stop_sx.subscribe();
            tokio::spawn(async move {
                info(format!(
                    "TCP Server acceptor thread {} start successfully.",
                    index
                ));
                loop {
                    select! {
                        val = stop_rx.recv() =>{
                            match val{
                                Ok(flag) => {
                                    if flag {
                                        info(format!("TCP Server acceptor thread {} stopped successfully.",index));
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

                                    if !establish_connection_check(&addr,&connection_manager,&mut write_frame_stream).await{
                                        continue;
                                    }

                                    let (connection_stop_sx, connection_stop_rx) = broadcast::channel::<bool>(1);
                                    let connection = NetworkConnection::new(
                                        crate::server::connection::NetworkConnectionType::TCP,
                                        addr,
                                        Some(connection_stop_sx.clone())
                                    );
                                    connection_manager.add_connection(connection.clone());
                                    connection_manager.add_tcp_write(connection.connection_id, write_frame_stream);

                                    read_frame_process(read_frame_stream,connection,request_queue_sx.clone(),connection_stop_rx);
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

    async fn handler_process(&self) {
        let mut request_queue_rx = self.request_queue_sx.subscribe();
        let response_queue_sx = self.response_queue_sx.clone();
        let command = self.command.clone();
        let connect_manager = self.connection_manager.clone();
        let stop_sx = self.stop_sx.clone();
        let handler_process_num = self.handler_process_num.clone();

        tokio::spawn(async move {
            let mut process_handler: HashMap<usize, Sender<RequestPackage>> = HashMap::new();
            handler_child_process(
                handler_process_num,
                stop_sx.clone(),
                response_queue_sx.clone(),
                connect_manager.clone(),
                command.clone(),
                &mut process_handler,
            );

            let mut stop_rx = stop_sx.subscribe();
            let mut process_handler_seq = 1;
            loop {
                select! {
                    val = stop_rx.recv() =>{
                        match val{
                            Ok(flag) => {
                                if flag {
                                    info("TCP Server handler thread stopped successfully.".to_string());
                                    break;
                                }
                            }
                            Err(_) => {}
                        }
                    },
                    val = request_queue_rx.recv()=>{
                        if let Ok(packet) = val{
                            metrics_request_queue("handler-total", request_queue_rx.len() as i64);

                            // Try to deliver the request packet to the child handler until it is delivered successfully. 
                            // Because some request queues may be full or abnormal, the request packets can be delivered to other child handlers.
                            loop{
                                let seq = if process_handler_seq > process_handler.len(){
                                    1
                                } else {
                                    process_handler_seq
                                };

                                if let Some(handler_sx) = process_handler.get(&seq){
                                    match handler_sx.send(packet.clone()){
                                        Ok(_) => {
                                            let lable = format!("handler-{}",seq);
                                            metrics_request_queue(&lable, handler_sx.len() as i64);
                                            break;
                                        }
                                        Err(err) => error(format!(
                                            "Failed to write data to the handler process queue, error message: {:?}",
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

    async fn response_process(&self) {
        let mut response_queue_rx = self.response_queue_sx.subscribe();
        let connect_manager = self.connection_manager.clone();
        let mut stop_rx = self.stop_sx.subscribe();
        let response_process_num = self.response_process_num.clone();
        let stop_sx = self.stop_sx.clone();

        tokio::spawn(async move {
            let mut process_handler = HashMap::new();
            for index in 1..=response_process_num {
                let (response_process_sx, mut response_process_rx) =
                    broadcast::channel::<ResponsePackage>(100);
                process_handler.insert(index, response_process_sx.clone());
                let mut raw_stop_rx = stop_sx.subscribe();
                let raw_connect_manager = connect_manager.clone();
                tokio::spawn(async move {
                    info(format!(
                        "TCP Server response process thread {} start successfully.",
                        index
                    ));

                    loop {
                        select! {
                            val = raw_stop_rx.recv() =>{
                                match val{
                                    Ok(flag) => {
                                        if flag {
                                            info(format!("TCP Server response process thread {} stopped successfully.",index));
                                            break;
                                        }
                                    }
                                    Err(_) => {}
                                }
                            },
                            val = response_process_rx.recv()=>{
                                if let Ok(response_package) = val{
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
                                        raw_connect_manager.clonse_connect(response_package.connection_id).await;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                });
            }

            info(format!(
                "TCP Server response process thread start successfully."
            ));
            let mut process_handler_seq = 1;
            loop {
                select! {
                    val = stop_rx.recv() =>{
                        match val{
                            Ok(flag) => {
                                if flag {
                                    info("TCP Server response process thread stopped successfully.".to_string());
                                    break;
                                }
                            }
                            Err(_) => {}
                        }
                    }

                    val = response_queue_rx.recv()=>{
                        if let Ok(packet) = val{
                            info(format!("response packet:{:?}", packet));
                            metrics_request_queue("response-total", response_queue_rx.len() as i64);

                            let seq = if process_handler_seq > process_handler.len(){
                                1
                            } else {
                                process_handler_seq
                            };

                            if let Some(handler_sx) = process_handler.get(&seq){
                                match handler_sx.send(packet){
                                    Ok(_) => {
                                        let lable = format!("response-{}",seq);
                                        metrics_request_queue(&lable, handler_sx.len() as i64);
                                    }
                                    Err(err) => error(format!(
                                        "Failed to write data to the handler process queue, error message: {:?}",
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
        });
    }

    async fn wait_stop_server(&self) {
        let mut stop_rx = self.stop_sx.subscribe();
        let connect_manager = self.connection_manager.clone();
        tokio::spawn(async move {
            loop {
                select! {
                    val = stop_rx.recv() =>{
                        match val{
                            Ok(flag) => {
                                if flag {
                                    connect_manager.close_all_connect().await;
                                    break;
                                }
                            }
                            Err(_) => {}
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
                    match val{
                        Ok(flag) => {
                            if flag {
                                info(format!("TCP connection 【{}】 acceptor thread stopped successfully.",connection.connection_id));
                                break;
                            }
                        }
                        Err(_) => {}
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
                                match request_queue_sx.send(package) {
                                    Ok(_) => {
                                        metrics_request_queue("handler-total", request_queue_sx.len() as i64);
                                    }
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
    response_queue_sx: Sender<ResponsePackage>,
    connection_manager: Arc<ConnectionManager>,
    command: Command<S>,
    process_handler: &mut HashMap<usize, Sender<RequestPackage>>,
) where
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    for index in 1..=handler_process_num {
        let (hendler_process_sx, mut hendler_process_rx) =
            broadcast::channel::<RequestPackage>(100);
        let mut raw_stop_rx = stop_sx.subscribe();
        let raw_response_queue_sx = response_queue_sx.clone();
        let raw_connect_manager = connection_manager.clone();
        let mut raw_command = command.clone();

        process_handler.insert(index, hendler_process_sx.clone());

        tokio::spawn(async move {
            info(format!(
                "TCP Server handler process thread {} start successfully.",
                index
            ));
            loop {
                select! {
                    val = raw_stop_rx.recv() =>{
                        match val{
                            Ok(flag) => {
                                if flag {
                                    info(format!("TCP Server handler process thread {} stopped successfully.",index));
                                    break;
                                }
                            }
                            Err(_) => {}
                        }
                    },
                    val = hendler_process_rx.recv()=>{
                        let lable = format!("handler-{}",index);
                        metrics_request_queue(&lable, hendler_process_sx.len() as i64);
                        if let Ok(packet) = val{


                            if let Some(connect) = raw_connect_manager.get_connect(packet.connection_id) {
                                if let Some(resp) = raw_command
                                    .apply(raw_connect_manager.clone(), connect, packet.addr, packet.packet)
                                    .await
                                {
                                    let response_package = ResponsePackage::new(packet.connection_id, resp);
                                    match raw_response_queue_sx.send(response_package) {
                                        Ok(_) => {
                                            metrics_request_queue("response-total", raw_response_queue_sx.len() as i64);
                                        }
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
