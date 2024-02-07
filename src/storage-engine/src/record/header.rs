use bytes::Bytes;

pub struct Header {
    pub key: String,
    pub value: Bytes,
}

impl Header {
    pub fn new(key: String, value: Bytes) -> Self {
        Header { key, value }
    }
}
