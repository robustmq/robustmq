#[cfg(test)]
mod tests {
    use common::config::meta::MetaConfig;
    use common::log;
    use std::thread::sleep;
    use std::time::{Duration, Instant};
    use meta::cluster::Cluster;
    use meta::raft::message::RaftMessage;
    use meta::{Meta, Node};
    use tokio::{sync::mpsc, time::timeout};

    #[test]
    fn running() {
        log::new();
        let conf = MetaConfig::default();
        let mut mt = Meta::new(conf);
        mt.start();

        loop {
            sleep(Duration::from_secs(1000));
        }
    }

    fn get_leader() -> Cluster {
        Cluster::new(Node::new("127.0.0.1".to_string(), 1))
    }

    #[tokio::test]
    async fn timeout_test() {
        let (raft_message_send, mut raft_message_recv) = mpsc::channel::<RaftMessage>(10000);
        let mut n = Instant::now();
        loop {
            match timeout(Duration::from_millis(100), raft_message_recv.recv()).await {
                Ok(_) => {}
                Err(err) => {}
            }
            if n.elapsed().as_millis() > 1000 {
                break;
            }
        }
    }
}
