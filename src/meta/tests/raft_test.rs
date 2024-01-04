#[cfg(test)]
mod tests {
    use common::config::meta::MetaConfig;
    use common::log;
    use meta::Meta;
    use std::thread::sleep;
    use std::time::Duration;
    use std::vec;

    #[test]
    fn running() {
        let mut conf = MetaConfig::default();
        conf.node_id = 1;
        conf.addr = "127.0.0.1".to_string();
        conf.port = 1220;
        conf.log_path = "/tmp/test_fold1/data".to_string();
        conf.data_path = "/tmp/test_fold1/logs".to_string();
        conf.meta_nodes = vec!["127.0.0.1:1220".to_string()];

        log::new(conf.log_path.clone(), 1024, 50);

        let mut mt = Meta::new(conf);
        mt.start();

        loop {
            sleep(Duration::from_secs(1000));
        }
    }
}
