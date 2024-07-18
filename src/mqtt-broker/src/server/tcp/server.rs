use super::{
    connection::NetworkConnection, connection_manager::ConnectionManager, packet::ResponsePackage,
};
use crate::{
    handler::{command::Command, validator::establish_connection_check},
    metrics::{metrics_request_queue, metrics_response_queue},
    server::tcp::packet::RequestPackage,
};
use common_base::{
    errors::RobustMQError,
    log::{debug, error, info},
};
use futures::StreamExt;
use protocol::mqtt::{
    codec::{MQTTPacketWrapper, MqttCodec},
    common::MQTTPacket,
};
use std::{collections::HashMap, sync::Arc};
use storage_adapter::storage::StorageAdapter;
use tokio::{io, select, sync::broadcast::Sender};
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
        max_connection_num: usize,
        handler_process_num: usize,
        response_process_num: usize,
        max_try_mut_times: u64,
        try_mut_sleep_time_ms: u64,
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

        for i in 1..=self.accept_thread_num {
            self.acceptor(arc_listener.clone(), i).await;
        }

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
        let arc_listener = Arc::new(listener);

        for i in 1..=self.accept_thread_num {
            self.acceptor(arc_listener.clone(), i).await;
        }

        self.handler_process().await;
        self.response_process().await;
        self.wait_stop_server().await;
    }

    async fn acceptor(&self, listener: Arc<TcpListener>, index: usize) {
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
                                let mut read_frame_stream = FramedRead::new(r_stream, codec.clone());
                                let mut  write_frame_stream = FramedWrite::new(w_stream, codec.clone());

                                let cm = connection_manager.clone();
                                let request_queue_sx = request_queue_sx.clone();

                                if !establish_connection_check(&addr,&cm,&mut write_frame_stream).await{
                                    continue;
                                }

                                let (connection_stop_sx,mut connection_stop_rx) = broadcast::channel::<bool>(1);
                                let connection_id = cm.add(
                                        NetworkConnection::new(
                                            crate::server::tcp::connection::NetworkConnectionType::TCP,
                                            addr,
                                            Some(connection_stop_sx.clone())
                                        )
                                 );
                                cm.add_write(connection_id, write_frame_stream);

                                tokio::spawn(async move {
                                    loop {
                                        select! {
                                            val = connection_stop_rx.recv() =>{
                                                match val{
                                                    Ok(flag) => {
                                                        if flag {
                                                            info(format!("TCP connection 【{}】 acceptor thread stopped successfully.",connection_id));
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
                                                            info(format!("revc packet:{:?}", pack));
                                                            let package =
                                                                RequestPackage::new(connection_id, addr, pack);
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
                            Err(e) => {
                                error(format!("TCP accept failed to create connection with error message :{:?}",e));
                            }
                        }
                    }
                };
            }
        });
    }

    async fn handler_process(&self) {
        let mut request_queue_rx = self.request_queue_sx.subscribe();
        let response_queue_sx = self.response_queue_sx.clone();
        let command = self.command.clone();
        let connect_manager = self.connection_manager.clone();
        let stop_sx = self.stop_sx.clone();
        let handler_process_num = self.handler_process_num.clone();

        tokio::spawn(async move {
            let mut process_handler = HashMap::new();
            for index in 1..=handler_process_num {
                let (hendler_process_sx, mut hendler_process_rx) =
                    broadcast::channel::<RequestPackage>(100);
                let mut raw_stop_rx = stop_sx.subscribe();
                let raw_response_queue_sx = response_queue_sx.clone();
                let raw_connect_manager = connect_manager.clone();
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

            let mut stop_rx = stop_sx.subscribe();
            let mut process_handler_seq = 1;
            info(format!(
                "TCP Server handler process thread start successfully."
            ));
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

                            let seq = if process_handler_seq > process_handler.len(){
                                1
                            } else {
                                process_handler_seq
                            };

                            if let Some(handler_sx) = process_handler.get(&seq){
                                match handler_sx.send(packet){
                                    Ok(_) => {
                                        let lable = format!("handler-{}",seq);
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
                                        raw_connect_manager
                                            .write_frame(response_package.connection_id, packet_wrapper)
                                            .await;
                                    }


                                    if let MQTTPacket::Disconnect(_, _) = response_package.packet {
                                        raw_connect_manager.clonse_connect(response_package.connection_id).await;
                                        continue;
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
