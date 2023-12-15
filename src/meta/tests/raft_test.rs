#[cfg(test)]
mod tests {
    use common::{config::meta::MetaConfig, runtime::create_runtime};

    use meta::raft::{message::RaftMessage, node::Node, raft::MetaRaft};
    use tokio::{sync::mpsc, runtime::Runtime};

    #[test]
    fn running() {
        let (raft_message_send, raft_message_recv) = mpsc::channel::<RaftMessage>(10000);
        let leader_node = get_leader();
        let mut config = MetaConfig::default();
        config.addr = "127.0.0.1".to_string();
        config.node_id = 1;
        config.data_path = "/tmp/data".to_string();
        let mut meta_raft = MetaRaft::new(config, leader_node, raft_message_recv);
        let runtime: Runtime = create_runtime("meta-test", 3);
        runtime.block_on(meta_raft.run());
        
    }

    fn get_leader() -> Node {
        Node::new("127.0.0.1".to_string(), 1)
    }
}
