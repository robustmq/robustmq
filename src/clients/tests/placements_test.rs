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

mod common;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use clients::placement::placement::call::{cluster_status, register_node};
    use protocol::placement_center::generate::placement::ClusterStatusRequest;

    use crate::common::get_placement_addr;
    use clients::poll::ClientPool;

    #[tokio::test]
    async fn placement_test() {
        let client_poll: Arc<ClientPool> = Arc::new(ClientPool::new(1));
        let addrs = vec![get_placement_addr()];

        let request=ClusterStatusRequest::default();

        match cluster_status(client_poll.clone(), addrs.clone(), request).await{
            Ok(_) => {}
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
}