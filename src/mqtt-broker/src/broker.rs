use crate::{grpc::server::GrpcServer, network::tcp_server::TcpServer};
use common::{config::server::RobustConfig, runtime::create_runtime, version::banner};
use flume::{Receiver, Sender};
use std::{
    fmt::Result, net::SocketAddr, sync::Arc, thread
};
use tokio::{io, time::error::Elapsed};

#[derive(Debug, thiserror::Error)]
#[error("Acceptor error")]
pub enum Error {
    #[error("I/O {0}")]
    Io(#[from] io::Error),
    #[error("Timeout")]
    Timeout(#[from] Elapsed),
}

pub struct Broker {
    config: Arc<RobustConfig>,
    signal_st: Sender<u16>,
    signal_rt: Receiver<u16>,
}

impl Broker {
    pub fn new(config: Arc<RobustConfig>) -> Broker {
        let (signal_st, signal_rt) = flume::bounded::<u16>(1);
        return Broker {
            config,
            signal_st,
            signal_rt,
        };
    }
    pub fn start(&self) -> Result {
        let mut thread_handles = Vec::new();

        // metrics init

        // Data flow requests are handled independently in a separate runtime
        let data_thread = thread::Builder::new().name("data-thread".to_owned());
        let config = self.config.clone();
        let data_thread_join = data_thread.spawn(move || {
            let data_runtime = create_runtime("data-runtime", config.runtime.data_worker_threads);
            data_runtime.block_on(async {
                let ip: SocketAddr = format!("{}:{}", config.addr, config.mqtt.mqtt4_port)
                    .parse()
                    .unwrap();
                let tcp_s = TcpServer::new(
                    ip,
                    config.network.accept_thread_num,
                    config.network.max_connection_num,
                    config.network.request_queue_size,
                    config.network.handler_thread_num,
                    config.network.response_queue_size,
                    config.network.response_thread_num,
                );
                tcp_s.start().await;
            });
        });
        thread_handles.push(data_thread_join);

        // Requests for cluster management and internal interaction classes are handled by a separate runtime
        let inner_thread = thread::Builder::new().name("http-thread".to_owned());
        let config = self.config.clone();
        let inner_thread_join = inner_thread.spawn(move || {
            let inner_runtime =
                create_runtime("inner-runtime", config.runtime.inner_worker_threads);

            // start Grpc Server
            let c = config.clone();
            inner_runtime.block_on(async move {
                let ip: SocketAddr = format!("{}:{}", c.addr, c.grpc_port).parse().unwrap();
                let g_s = GrpcServer::new(ip);
                g_s.start().await;
            });
            
        });
        thread_handles.push(inner_thread_join);

        // process start hook
        banner();

        thread_handles.into_iter().for_each(|handle| {
            // join() might panic in case the thread panics
            // we just ignore it
            let _ = handle.unwrap().join();
        });

        return Ok(());
    }

    pub fn stop(&self) -> Result {
        // Recovery of resources

        // Sends a signal to stop the process
        self.signal_st.send(1).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};
    use common::runtime::create_runtime;
    use std::thread::sleep;
    use std::time::Duration;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
    };

    #[test]
    fn start_broker() {
        let rt = create_runtime("text", 10);
        let guard = rt.enter();
        // let config = MetaConfig::default();
        // let b = Broker::new(config);
        // b.start();
        drop(guard);
    }

    #[test]
    fn client() {
        let rt = create_runtime("text", 10);
        let guard = rt.enter();
        tokio::spawn(async move {
            let mut stream = TcpStream::connect("127.0.0.1:8768").await.unwrap();
            // let mut stream = Framed::new(stream, LengthDelimitedCodec::new());
            let mut write_buf = BytesMut::with_capacity(20);
            write_buf.put(&b"hello world lobo"[..]);
            let _ = stream.write_all(&write_buf).await;

            let mut read_buf = BytesMut::with_capacity(20);
            match stream.read_buf(&mut read_buf).await {
                Ok(_) => {
                    let content = String::from_utf8_lossy(&read_buf).to_string();
                    println!("receive:{}", content)
                }
                Err(err) => {
                    println!("err:{:?}", err)
                }
            }
        });
        drop(guard);
        sleep(Duration::from_secs(10));
    }
}
