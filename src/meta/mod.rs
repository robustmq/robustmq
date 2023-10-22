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

use crate::config::RobustConfig;
use crate::log;
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use tokio::{net::TcpListener, runtime::Runtime};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

mod server;

pub struct MetaServer<'a> {
    config: &'a RobustConfig,
}

impl<'a> MetaServer<'a> {
    pub fn new(config: &'a RobustConfig) -> Self {
        return MetaServer { config };
    }

    pub fn start(&self) -> Runtime {
        let runtime: Runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(self.config.broker.work_thread.unwrap() as usize)
            .max_blocking_threads(2048)
            .thread_name("meta-http")
            .enable_io()
            .build()
            .unwrap();

        let _gurad = runtime.enter();
        let ip = format!("{}:{}", self.config.addr, self.config.broker.port.unwrap());

        runtime.spawn(async {
            log::info(&format!("http server start success. bind:{}", ip));
            let listener = TcpListener::bind(ip).await.unwrap();
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let mut stream = Framed::new(stream, LengthDelimitedCodec::new());
                tokio::spawn(async move {
                    while let Some(Ok(data)) = stream.next().await {
                        println!("Got: {:?}", String::from_utf8_lossy(&data));

                        // 发送的消息也只需要发送消息主体，不需要提供长度
                        // Framed/LengthDelimitedCodec 会自动计算并添加
                        //    let response = &data[0..5];
                        stream.send(Bytes::from(data)).await.unwrap();
                    }
                });
            }
        });
        return runtime;
    }
}
