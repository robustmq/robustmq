use common_base::config::journal_server::journal_server_conf;

use self::tcp::tcp_server::TcpServer;

pub mod quic;
pub mod tcp;

pub async fn start_tcp_server() {
    let conf = journal_server_conf();
    let tcp = TcpServer::new(
        conf.network.accept_thread_num,
        conf.network.max_connection_num,
        conf.network.request_queue_size,
        conf.network.handler_thread_num,
        conf.network.response_queue_size,
        conf.network.response_thread_num,
        60,
        10,
    );
    tcp.start(conf.grpc_port).await;
}

pub async fn start_quic_server() {
    let conf = journal_server_conf();
}
