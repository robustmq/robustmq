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

    use crate::journal_client::common::journal_tcp_addr_vec;

    #[tokio::test]
    async fn create_shard_test() {
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

        let data = JournalClientWriteData {
            key: "k1".to_string(),
            content: "ccccc1".as_bytes().to_vec(),
            tags: vec!["tag1".to_string()],
        };

        let res_opt = client
            .write(namespace.clone(), shard_name.clone(), data)
            .await;
        assert!(res_opt.is_ok());

        let res = res_opt.unwrap();
        assert_eq!(res.offset, 0);

        let data = JournalClientWriteData {
            key: "k2".to_string(),
            content: "ccccc2".as_bytes().to_vec(),
            tags: vec!["tag2".to_string()],
        };
        let res_opt = client
            .write(namespace.clone(), shard_name.clone(), data)
            .await;
        assert!(res_opt.is_ok());

        let res = res_opt.unwrap();
        assert_eq!(res.offset, 0);

        let datas = vec![
            JournalClientWriteData {
                key: "k2".to_string(),
                content: "ccccc2".as_bytes().to_vec(),
                tags: vec!["tag2".to_string()],
            },
            JournalClientWriteData {
                key: "k2".to_string(),
                content: "ccccc2".as_bytes().to_vec(),
                tags: vec!["tag2".to_string()],
            },
            JournalClientWriteData {
                key: "k2".to_string(),
                content: "ccccc2".as_bytes().to_vec(),
                tags: vec!["tag2".to_string()],
            },
        ];
        let res_opt = client
            .batch_write(namespace.clone(), shard_name.clone(), datas)
            .await;
        assert!(res_opt.is_ok());

        let res = res_opt.unwrap();
        println!("{:?}", res);
        assert_eq!(res.len(), 3);

        assert_eq!(res.get(0).unwrap().offset, 2);
        assert_eq!(res.get(1).unwrap().offset, 3);
        assert_eq!(res.get(2).unwrap().offset, 4);

        let res = client.close().await;
        println!("{:?}", res);
        assert!(res.is_ok());
    }
}
