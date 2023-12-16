#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};
    use std::thread::sleep;
    use common::{config::meta::MetaConfig, runtime::create_runtime};

    use meta::raft::{message::RaftMessage, node::Node, raft::MetaRaft};
    use tokio::{runtime::Runtime, sync::mpsc, time::timeout};

    #[test]
    fn running() {
        let (raft_message_send, raft_message_recv) = mpsc::channel::<RaftMessage>(10000);
        let leader_node = get_leader();
        let mut config = MetaConfig::default();
        config.data_path = "/tmp/data".to_string();
        let mut meta_raft = MetaRaft::new(config, raft_message_recv);
        let runtime: Runtime = create_runtime("meta-test", 3);
        runtime.block_on(async {
            meta_raft.ready().await;
        });

        loop {
            sleep(Duration::from_secs(5));
        }
    }

    fn get_leader() -> Node {
        Node::new("127.0.0.1".to_string(), 1)
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
