use super::connection::Connection;

#[derive(Debug)]
pub struct RequestPackage {
    pub content: String,
    pub conn:Connection,
}

impl RequestPackage {
    pub fn new(content: String,conn: Connection) -> Self {
        Self { content ,conn}
    }
}

#[derive(Debug)]
pub struct ResponsePackage {
    pub content: String,
}

impl ResponsePackage {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}
