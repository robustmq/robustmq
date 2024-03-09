#[cfg(test)]
mod tests {
    use byteorder::{BigEndian, ReadBytesExt};
    use common::config::placement_center::init_placement_center_conf_by_config;
    use common::log::{self, init_placement_center_log};
    use common::{config::placement_center::PlacementCenterConfig, tools::create_fold};
    use placement_center::PlacementCenter;
    use prost::Message;
    use raft::eraftpb::Entry;
    use std::io::Cursor;
    use std::vec;
    use tokio::sync::broadcast;
    use toml::Table;

    #[test]
    fn raft_node_1() {
        let mut conf = PlacementCenterConfig::default();
        conf.node_id = 1;
        conf.addr = "127.0.0.1".to_string();
        conf.grpc_port = 1221;
        conf.http_port = 2221;
        conf.log_path = "/tmp/test_fold1/logs".to_string();
        conf.data_path = "/tmp/test_fold1/data".to_string();

        let mut nodes = Table::new();
        nodes.insert(
            1.to_string(),
            toml::Value::String("127.0.0.1:1221".to_string()),
        );
        nodes.insert(
            2.to_string(),
            toml::Value::String("127.0.0.1:1222".to_string()),
        );
        nodes.insert(
            2.to_string(),
            toml::Value::String("127.0.0.1:1223".to_string()),
        );
        conf.nodes = nodes;
        init_placement_center_conf_by_config(conf);
        init_placement_center_log();

        let (stop_send, _) = broadcast::channel(10);
        let mut mt = PlacementCenter::new();

        mt.start(stop_send, false);
    }

    #[test]
    fn raft_node_2() {
        let mut conf = PlacementCenterConfig::default();
        conf.node_id = 2;
        conf.addr = "127.0.0.1".to_string();
        conf.grpc_port = 1222;
        conf.http_port = 2222;
        conf.log_path = "/tmp/test_fold2/logs".to_string();
        conf.data_path = "/tmp/test_fold2/data".to_string();
        create_fold(conf.data_path.clone());
        create_fold(conf.log_path.clone());

        let mut nodes = Table::new();
        nodes.insert(
            1.to_string(),
            toml::Value::String("127.0.0.1:1221".to_string()),
        );
        nodes.insert(
            2.to_string(),
            toml::Value::String("127.0.0.1:1222".to_string()),
        );
        nodes.insert(
            2.to_string(),
            toml::Value::String("127.0.0.1:1223".to_string()),
        );
        conf.nodes = nodes;
        init_placement_center_conf_by_config(conf);
        init_placement_center_log();

        let (stop_send, _) = broadcast::channel(10);
        let mut mt = PlacementCenter::new();

        mt.start(stop_send, false);
    }

    #[test]
    fn raft_node_3() {
        let mut conf = PlacementCenterConfig::default();
        conf.node_id = 3;
        conf.addr = "127.0.0.1".to_string();
        conf.grpc_port = 1223;
        conf.http_port = 2223;
        conf.log_path = "/tmp/test_fold3/logs".to_string();
        conf.data_path = "/tmp/test_fold3/data".to_string();

        let mut nodes = Table::new();
        nodes.insert(
            1.to_string(),
            toml::Value::String("127.0.0.1:1221".to_string()),
        );
        nodes.insert(
            2.to_string(),
            toml::Value::String("127.0.0.1:1222".to_string()),
        );
        nodes.insert(
            2.to_string(),
            toml::Value::String("127.0.0.1:1223".to_string()),
        );
        conf.nodes = nodes;

        init_placement_center_conf_by_config(conf);
        init_placement_center_log();

        let (stop_send, _) = broadcast::channel(10);
        let mut mt = PlacementCenter::new();

        mt.start(stop_send, false);
    }

    #[test]
    fn vec_test() {
        let v = vec![1, 2, 3, 4, 5, 6];
        let start = 0 as usize;
        let end = 3 as usize;
        println!("{:?}", v[start..end].to_vec());
        println!("{:?}", v[start..end].to_vec());
    }

    #[test]
    fn byte_order_test() {
        let mut rdr = Cursor::new(vec![2, 5, 3, 0]);
        let v = rdr.read_u16::<BigEndian>().unwrap();
        println!("{}", v);

        // let mut wtr = vec![];
        // wtr.write_u16::<LittleEndian>(64).unwrap();
        // let mut rdr = Cursor::new(wtr);
        // let v = rdr.read_u64::<BigEndian>().unwrap();
        // println!("{}", v);

        let v1 = "666".to_string().into_bytes();
        println!("{:?}", v1);
        println!("{:?}", String::from_utf8(v1).unwrap());

        let v2 = 666u64.encode_to_vec();

        let v2 = 666u64.to_be_bytes();
        println!("{}", u64::from_be_bytes(v2));
    }

    #[test]
    fn entry_vec_test() {
        let mut entries = Vec::new();
        let mut e1 = Entry::default();
        e1.set_index(1);
        entries.push(e1);

        let mut e2 = Entry::default();
        e2.set_index(2);
        entries.push(e2);

        let mut res_entries = Vec::new();
        res_entries.extend_from_slice(&entries);
        println!("{:?}", res_entries);

        res_entries.drain(1..);
        println!("{:?}", res_entries);

        let mut entries = Vec::new();
        let mut e1 = Entry::default();
        e1.set_index(2);
        entries.push(e1);

        let mut e2 = Entry::default();
        e2.set_index(4);
        entries.push(e2);
        res_entries.extend_from_slice(&entries);
        println!("{:?}", res_entries);
    }
}
