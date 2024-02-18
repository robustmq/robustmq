pub struct Shard {
    pub shard_id: String,
    pub shard_name: String,
    pub start_segment: u64,
    pub end_segment: u64,
    pub segment_num: u64,
    pub start_offset: u64,
    pub end_offset: u64,
    pub create_time: u64,
}

impl Shard {
    pub fn create_segment(){

    }

    pub fn delete_segment(){
        
    }
}
