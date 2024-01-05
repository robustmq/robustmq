#[cfg(test)]
mod tests {
    use common::config::meta::MetaConfig;
    use common::log;
    use meta::Meta;
    use std::thread::sleep;
    use std::time::Duration;
    use std::vec;

    #[test]
    fn raft_node_1() {
        let mut conf = MetaConfig::default();
        conf.node_id = 1;
        conf.addr = "127.0.0.1".to_string();
        conf.port = 1221;
        conf.log_path = "/tmp/test_fold1/data".to_string();
        conf.data_path = "/tmp/test_fold1/logs".to_string();
        conf.meta_nodes = vec!["127.0.0.1:1221".to_string(),"127.0.0.1:1222".to_string(),"127.0.0.1:1223".to_string()];

        log::new(conf.log_path.clone(), 1024, 50);

        let mut mt = Meta::new(conf);
        mt.start();

        loop {
            sleep(Duration::from_secs(1000));
        }
    }

    #[test]
    fn raft_node_2() {
        let mut conf = MetaConfig::default();
        conf.node_id = 2;
        conf.addr = "127.0.0.1".to_string();
        conf.port = 1222;
        conf.log_path = "/tmp/test_fold2/data".to_string();
        conf.data_path = "/tmp/test_fold2/logs".to_string();
        conf.meta_nodes = vec!["127.0.0.1:1221".to_string(),"127.0.0.1:1222".to_string(),"127.0.0.1:1223".to_string()];

        log::new(conf.log_path.clone(), 1024, 50);

        let mut mt = Meta::new(conf);
        mt.start();

        loop {
            sleep(Duration::from_secs(1000));
        }
    }

    #[test]
    fn raft_node_3() {
        let mut conf = MetaConfig::default();
        conf.node_id = 3;
        conf.addr = "127.0.0.3".to_string();
        conf.port = 1223;
        conf.log_path = "/tmp/test_fold3/data".to_string();
        conf.data_path = "/tmp/test_fold3/logs".to_string();
        conf.meta_nodes = vec!["127.0.0.1:1221".to_string(),"127.0.0.1:1222".to_string(),"127.0.0.1:1223".to_string()];

        log::new(conf.log_path.clone(), 1024, 50);

        let mut mt = Meta::new(conf);
        mt.start();

        loop {
            sleep(Duration::from_secs(1000));
        }
    }
}
