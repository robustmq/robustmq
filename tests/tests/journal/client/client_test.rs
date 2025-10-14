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
    use metadata_struct::adapter::read_config::ReadConfig;

    use crate::journal::client::common::journal_tcp_addr_vec;

    #[ignore]
    #[tokio::test]
    async fn write_data_test() {
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

        write_data(&client, &namespace, &shard_name).await;
    }

    #[ignore]
    #[tokio::test]
    async fn read_data_test() {
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

        write_data(&client, &namespace, &shard_name).await;

        // read by key
        let read_config = ReadConfig::new();
        let res = client
            .read_by_key(&namespace, &shard_name, 0, "k1", &read_config)
            .await;
        println!("{res:?}");
        assert!(res.is_ok());
        let list = res.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list.first().unwrap().key, "k1".to_string());

        // read by key
        let read_config = ReadConfig::new();
        let res = client
            .read_by_key(&namespace, &shard_name, 3, "k1", &read_config)
            .await;
        println!("{res:?}");
        assert!(res.is_ok());
        let list = res.unwrap();
        assert_eq!(list.len(), 0);

        // read by tag
        let read_config = ReadConfig::new();
        let res = client
            .read_by_tag(&namespace, &shard_name, 0, "tag1", &read_config)
            .await;
        println!("{res:?}");
        assert!(res.is_ok());
        let list = res.unwrap();
        assert_eq!(list.len(), 1);
        assert!(list.first().unwrap().tags.contains(&("tag1".to_string())));

        // read by tag
        let read_config = ReadConfig::new();
        let res = client
            .read_by_tag(&namespace, &shard_name, 3, "tag1", &read_config)
            .await;
        println!("{res:?}");
        assert!(res.is_ok());
        let list = res.unwrap();
        assert_eq!(list.len(), 0);

        // read by offset
        let read_config = ReadConfig::new();
        let res = client
            .read_by_offset(&namespace, &shard_name, 0, &read_config)
            .await;
        println!("{res:?}");
        assert!(res.is_ok());
        let list = res.unwrap();
        assert_eq!(list.len(), 5);

        // read by offset
        let read_config = ReadConfig::new();
        let res = client
            .read_by_offset(&namespace, &shard_name, 3, &read_config)
            .await;
        println!("{res:?}");
        assert!(res.is_ok());
        let list = res.unwrap();
        assert_eq!(list.len(), 2);
    }

    async fn write_data(client: &JournalClient, namespace: &str, shard_name: &str) {
        let data = JournalClientWriteData {
            key: "k0".to_string(),
            content: "ccccc0".as_bytes().to_vec(),
            tags: vec!["tag0".to_string()],
        };

        let res_opt = client
            .write(namespace.to_owned(), shard_name.to_owned(), data)
            .await;
        assert!(res_opt.is_ok());

        let res = res_opt.unwrap();
        println!("{res:?}");
        assert!(res.is_ok());
        assert_eq!(res.offset, 0);

        let data = JournalClientWriteData {
            key: "k1".to_string(),
            content: "ccccc1".as_bytes().to_vec(),
            tags: vec!["tag1".to_string()],
        };
        let res_opt = client
            .write(namespace.to_owned(), shard_name.to_owned(), data)
            .await;
        assert!(res_opt.is_ok());

        let res = res_opt.unwrap();
        println!("{res:?}");
        assert!(res.is_ok());
        assert_eq!(res.offset, 1);

        let data = vec![
            JournalClientWriteData {
                key: "k2".to_string(),
                content: "ccccc2".as_bytes().to_vec(),
                tags: vec!["tag2".to_string()],
            },
            JournalClientWriteData {
                key: "k3".to_string(),
                content: "ccccc3".as_bytes().to_vec(),
                tags: vec!["tag3".to_string()],
            },
            JournalClientWriteData {
                key: "k4".to_string(),
                content: "ccccc4".as_bytes().to_vec(),
                tags: vec!["tag4".to_string()],
            },
        ];
        let res_opt = client
            .batch_write(namespace.to_owned(), shard_name.to_owned(), data)
            .await;
        assert!(res_opt.is_ok());

        let res = res_opt.unwrap();
        assert_eq!(res.len(), 3);
        for row in res.clone() {
            assert!(row.is_ok());
        }
        assert_eq!(res.first().unwrap().offset, 2);
        assert_eq!(res.get(1).unwrap().offset, 3);
        assert_eq!(res.get(2).unwrap().offset, 4);

        let res = client.close().await;
        assert!(res.is_ok());
    }
}
