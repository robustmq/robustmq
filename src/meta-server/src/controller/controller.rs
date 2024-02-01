use common::log::info_meta;

pub struct Controller {}

impl Controller {
    pub fn new() -> Self {
        Controller {}
    }

    pub async fn start(&self) {
        info_meta("Cluster controller started successfully");
    }

    pub async fn stop(&self) {
        info_meta("Cluster controller stop");
    }
}
