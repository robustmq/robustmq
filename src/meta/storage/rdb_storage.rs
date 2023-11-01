use raft::Storage as RaftStorage;

struct RaftRocksDBStorage{

}

impl RaftRocksDBStorage{
    pub fn new() -> Self{
        return RaftRocksDBStorage{

        };
    }
}

impl RaftStorage for RaftRocksDBStorage{
    fn initial_state(&self) -> raft::Result<raft::RaftState> {
        
    }

}

#[cfg(test)]
mod tests{
    fn raft_node(){

    }
}