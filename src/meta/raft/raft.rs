use crate::config::meta::MetaConfig;
use crate::meta::raft::storage::RaftRocksDBStorage;
use raft::Config;
use raft::RawNode;
use slog_async;
use slog::o;
use slog::Drain;
struct RaftServer<'a>{
    conf: &'a MetaConfig,
}

impl<'a> RaftServer<'a>{

    pub fn new(conf:&'a MetaConfig) -> Self{
        return RaftServer { conf: conf };
    }

    pub fn start(&self) {
        let storage = RaftRocksDBStorage::new(self.conf);

        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain)
            .chan_size(4096)
            .overflow_strategy(slog_async::OverflowStrategy::Block)
            .build()
            .fuse();
        let logger = slog::Logger::root(drain, o!("tag" => format!("[{}]", 1)));


        let cfg = Config{
            id:1,
            election_tick:10,
            heartbeat_tick: 3,
            max_size_per_msg: 1024*1024*1024,
            max_inflight_msgs:256,
            applied:0,
            ..Default::default()
        };

        let mut node = RawNode::new(&cfg, storage, &logger);
    }
}