#[derive(Debug)]
pub struct RequestPackage {
    pub content: String,
}

impl RequestPackage {
    pub fn new(content: String) -> Self {
        Self { content }
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
