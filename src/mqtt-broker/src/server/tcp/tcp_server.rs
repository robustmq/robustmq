use super::{
    connection::Connection, connection_manager::ConnectionManager, packet::ResponsePackage,
};
use crate::{
    handler::command::Command,
    metrics::{
        metrics_request_packet_incr, metrics_request_queue, metrics_response_packet_incr,
        metrics_response_queue,
    },
    server::{tcp::packet::RequestPackage, MQTTProtocol},
};
use common_base::log::{debug, error, info};
use futures::StreamExt;
use protocol::mqtt::{DisconnectReasonCode, MQTTPacket};
use std::{
    fmt::{Debug, Error},
    sync::Arc,
};
use storage_adapter::storage::StorageAdapter;
use tokio::net::TcpListener;
use tokio::{io, sync::broadcast::Sender};
use tokio_util::codec::{Decoder, Encoder, FramedRead, FramedWrite};

// U: codec: encoder + decoder
// T: metadata storage adapter
// S: message storage adapter
pub struct TcpServer<U, T, S> {
    protocol: MQTTProtocol,
    command: Command<T, S>,
    connection_manager: Arc<ConnectionManager<U>>,
    accept_thread_num: usize,
    handler_process_num: usize,
    response_process_num: usize,
    request_queue_sx: Sender<RequestPackage>,
    response_queue_sx: Sender<ResponsePackage>,
    codec: U,
}

impl<U, T, S> TcpServer<U, T, S>
where
    U: Clone + Decoder + Encoder<MQTTPacket> + Send + Sync + 'static + Debug,
    MQTTPacket: From<<U as tokio_util::codec::Decoder>::Item>,
    <U as tokio_util::codec::Encoder<MQTTPacket>>::Error: Debug,
    <U as tokio_util::codec::Decoder>::Error: Debug,
    T: StorageAdapter + Clone + Send + Sync + 'static,
    S: StorageAdapter + Clone + Send + Sync + 'static,
{
    pub fn new(
        protocol: MQTTProtocol,
        command: Command<T, S>,
        accept_thread_num: usize,
        max_connection_num: usize,
        handler_process_num: usize,
        response_process_num: usize,
        max_try_mut_times: u64,
        try_mut_sleep_time_ms: u64,
        codec: U,
        request_queue_sx: Sender<RequestPackage>,
        response_queue_sx: Sender<ResponsePackage>,
    ) -> Self {
        let connection_manager = Arc::new(ConnectionManager::<U>::new(
            protocol.clone(),
            max_connection_num,
            max_try_mut_times,
            try_mut_sleep_time_ms,
        ));

        Self {
            protocol,
            command,
            connection_manager,
            accept_thread_num,
            handler_process_num,
            response_process_num,
            request_queue_sx,
            response_queue_sx,
            codec,
        }
    }

    pub async fn start(&self, port: u32) {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .unwrap();
        let arc_listener = Arc::new(listener);

        for _ in 1..=self.accept_thread_num {
            _ = self.acceptor(arc_listener.clone()).await;
        }

        for _ in 1..=self.handler_process_num {
            _ = self.handler_process().await;
        }

        for _ in 1..=self.response_process_num {
            _ = self.response_process().await;
        }
    }

    async fn acceptor(&self, listener: Arc<TcpListener>) -> Result<(), Error> {
        let request_queue_sx = self.request_queue_sx.clone();
        let connection_manager = self.connection_manager.clone();
        let codec = self.codec.clone();
        let protocol: String = self.protocol.clone().into();
        tokio::spawn(async move {
            loop {
                let cm = connection_manager.clone();
                let request_queue_sx = request_queue_sx.clone();
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        // split stream

                        let (r_stream, w_stream) = io::split(stream);
                        let mut read_frame_stream = FramedRead::new(r_stream, codec.clone());
                        let write_frame_stream = FramedWrite::new(w_stream, codec.clone());

                        // connect check
                        match cm.connect_check() {
                            Ok(_) => {}
                            Err(e) => {
                                error(format!("tcp connection failed to establish from IP: {}. Failure reason: {}",addr.to_string(),e.to_string()));
                                continue;
                            }
                        }

                        // connection manager
                        let connection_id = cm.add(Connection::new(addr));
                        cm.add_write(connection_id, write_frame_stream);
                        let protocol_lable = protocol.clone();

                        // request is processed by a separate thread, placing the request packet in the request queue.
                        tokio::spawn(async move {
                            loop {
                                if let Some(pkg) = read_frame_stream.next().await {
                                    match pkg {
                                        Ok(data) => {
                                            metrics_request_packet_incr(&protocol_lable);
                                            let pack: MQTTPacket = data.try_into().unwrap();
                                            let package = RequestPackage::new(connection_id, pack);
                                            match request_queue_sx.send(package) {
                                                Ok(_) => {}
                                                Err(err) => error(format!("Failed to write data to the request queue, error message: {:?}",err)),
                                            }
                                        }
                                        Err(e) => {
                                            debug(format!(
                                                "read_frame_stream decode error,error info: {:?}",
                                                e
                                            ));
                                        }
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        error(e.to_string());
                    }
                };
            }
        });

        return Ok(());
    }

    async fn handler_process(&self) -> Result<(), Error> {
        let mut request_queue_rx = self.request_queue_sx.subscribe();
        let response_queue_sx = self.response_queue_sx.clone();
        let protocol_lable: String = self.protocol.clone().into();
        let mut command = self.command.clone();
        let connect_manager = self.connection_manager.clone();
        tokio::spawn(async move {
            while let Ok(packet) = request_queue_rx.recv().await {
                metrics_request_queue(&protocol_lable, response_queue_sx.len() as i64);

                if let Some(resp) = command.apply(packet.connection_id, packet.packet).await {
                    // Close the client connection
                    if let MQTTPacket::Disconnect(disconnect, _) = resp.clone() {
                        if disconnect.reason_code == DisconnectReasonCode::NormalDisconnection {
                            connect_manager.clonse_connect(packet.connection_id).await;
                            continue;
                        }
                    }

                    // Writes the result of the business logic processing to the return queue
                    let response_package = ResponsePackage::new(packet.connection_id, resp);
                    match response_queue_sx.send(response_package) {
                        Ok(_) => {}
                        Err(err) => error(format!(
                            "Failed to write data to the response queue, error message: {:?}",
                            err
                        )),
                    }
                }
            }
        });
        return Ok(());
    }

    async fn response_process(&self) -> Result<(), Error> {
        let mut response_queue_rx = self.response_queue_sx.subscribe();
        let connect_manager = self.connection_manager.clone();
        let protocol_lable: String = self.protocol.clone().into();

        tokio::spawn(async move {
            while let Ok(response_package) = response_queue_rx.recv().await {
                metrics_response_queue(&protocol_lable, response_queue_rx.len() as i64);
                metrics_response_packet_incr(&protocol_lable);

                info(format!(
                    "response packet:{:?}",
                    response_package.packet.clone()
                ));
                connect_manager
                    .write_frame(response_package.connection_id, response_package.packet)
                    .await;
            }
        });
        return Ok(());
    }
}
