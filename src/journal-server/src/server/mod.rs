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
