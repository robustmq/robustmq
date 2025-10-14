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
    use common_base::tools::unique_id;
    use journal_client::client::{JournalClient, JournalClientWriteData};

    use crate::journal::client::common::journal_tcp_addr_vec;

    #[ignore]
    #[tokio::test]
    async fn write_scroll_test() {
        let addrs = journal_tcp_addr_vec();
        let namespace = unique_id();
        let shard_name = "s1".to_string();
        let replica_num = 1;

        let res = JournalClient::new(addrs).await;
        assert!(res.is_ok());
        let client = res.unwrap();

        let res = client
            .create_shard(&namespace, &shard_name, replica_num)
            .await;
        assert!(res.is_ok());

        for i in 0..100 {
            let content = format!("x{i}").repeat(1024);
            let data = JournalClientWriteData {
                key: format!("key-{i}"),
                content: content.as_bytes().to_vec(),
                tags: vec![format!("tag-{}", i)],
            };

            let res_opt = client
                .write(namespace.to_owned(), shard_name.to_owned(), data)
                .await;
            assert!(res_opt.is_ok());
        }
    }
}
