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

use std::io::Error;

use tokio::net::TcpListener;
use tokio_util::codec::{LengthDelimitedCodec, Framed};

pub async fn start(addr: String, port: Option<u16>, worker_threads: usize) -> Result<(),Error>{
    // let runtime = tokio::runtime::Builder::new_current_thread()
    //     .worker_threads(worker_threads)
    //     .max_blocking_threads(2048)
    //     .thread_name("admin-http")
    //     .enable_io()
    //     .build()
    //     .unwrap();
    // let listener = TcpListener::bind(format!("{}:{}", addr, port.unwrap())).await?;
    // let (stream, addr) = listener.accept().await?;
    // let mut stream = Framed::new(stream, LengthDelimitedCodec::new());
    // runtime.spawn(async move{
    //    while let Some(Ok(Data)) = stream.next().await{
    //         stream.send(Bytes::from(data)).await.unw;
    //    }
    // });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
  
    }
}
