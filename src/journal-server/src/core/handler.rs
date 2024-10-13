#[derive(Debug, Clone)]
pub struct Handler {}

impl Handler {
    pub fn new() -> Handler {
        return Handler {};
    }

    pub async fn write(&self) {}

    pub async fn read(&self) {}

    pub async fn active_segment(&self) {}

    pub async fn offset_commit(&self) {}
}
