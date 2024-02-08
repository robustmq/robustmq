pub struct Segment {
    pub shard_id: String,
    pub shard_name: String,
    pub segment_seq: u64,
    pub start_offset: u64,
    pub end_offset: u64,
    pub replia: Vec<u64>,
    pub leader: u64,
}

impl Segment {
    
}
