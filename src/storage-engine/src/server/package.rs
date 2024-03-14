#[derive(Debug)]
pub struct RequestPackage {
    pub content_size: usize,
    pub content: String,
    pub connection_id: u64,
}

impl RequestPackage {
    pub fn new(content_size: usize, content: String, connection_id: u64) -> Self {
        Self {
            content_size,
            content,
            connection_id,
        }
    }
}

#[derive(Debug)]
pub struct ResponsePackage {
    pub content: String,
    pub connection_id: u64,
}

impl ResponsePackage {
    pub fn new(content: String, connection_id: u64) -> Self {
        Self {
            content,
            connection_id,
        }
    }
}
