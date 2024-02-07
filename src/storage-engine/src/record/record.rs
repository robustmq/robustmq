use bytes::Bytes;

use super::header::Header;

pub struct Record {
    pub offset: u64,
    pub sequence: u32,
    pub timestamp: u64,
    pub size: u32,
    pub is_compressed: bool,
    pub has_header: bool,
    pub headers: Vec<Header>,
    pub key_size: u32,
    pub key: Bytes,
    pub value_size: u32,
    pub value: Bytes,
}
