use std::time::Duration;

use tokio::time::sleep;

pub struct PushServer {}

impl PushServer {
    pub fn new() -> Self {
        return PushServer {};
    }

    pub async fn start(&self) {
        loop {
            sleep(Duration::from_secs(5)).await;
        }
    }
}
