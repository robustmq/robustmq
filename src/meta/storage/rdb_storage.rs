use raft::Storage as RaftStorage;
use raft::Result as RaftResult;

struct RaftRocksDBStorage{

}

impl RaftRocksDBStorage{
    pub fn new() -> Self{
        return RaftRocksDBStorage{

        };
    }
}

impl RaftStorage for RaftRocksDBStorage{
    fn initial_state(&self) -> RaftResult<raft::RaftState> {
        
    }

    fn entries(
            &self,
            low: u64,
            high: u64,
            max_size: impl Into<Option<u64>>,
            context: raft::GetEntriesContext,
        ) -> RaftResult<Vec<Entry>> {
        
    }

    fn first_index(&self) -> RaftResult<u64> {
        
    }

    fn last_index(&self) -> RaftResult<u64> {
        
    }

    fn snapshot(&self, request_index: u64, to: u64) -> RaftResult<Snapshot> {
        
    }

    fn term(&self, idx: u64) -> RaftResult<u64> {
        
    }
}

#[cfg(test)]
mod tests{
    fn raft_node(){

    }
}