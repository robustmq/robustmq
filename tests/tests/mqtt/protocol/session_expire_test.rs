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

#[cfg(test)]
mod tests {
    use common_base::tools::now_second;
    use std::time::Duration;
    use tokio::time::sleep;

    use crate::mqtt::protocol::{
        common::{
            broker_addr_by_type, build_client_id, connect_server, distinct_conn,
            session_expiry_interval, wait_for_session_count_by_admin,
        },
        ClientTestProperties,
    };

    #[tokio::test]
    async fn session_expire_test() {
        let network = "tcp";
        let client_id = build_client_id(format!("session_expire_test_{network}").as_str());
        let client_properties = ClientTestProperties {
            mqtt_version: 5,
            client_id: client_id.to_string(),
            addr: broker_addr_by_type(network),
            ..Default::default()
        };
        let cli = connect_server(&client_properties);
        sleep(Duration::from_millis(100)).await;
        distinct_conn(cli);

        let start_time = now_second();
        wait_for_session_count_by_admin(&client_id, 0, Duration::from_secs(180)).await;
        let total = now_second() - start_time;
        println!("total:{total}");
        let sei = session_expiry_interval() as u64;
        assert!(total >= (sei - 2) && total <= (sei + 2));
    }
}
