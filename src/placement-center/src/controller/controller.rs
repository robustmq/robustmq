use common::log::info_meta;

pub struct StorageEneineController {}

impl StorageEneineController {
    pub fn new() -> Self {
        StorageEneineController {}
    }

    pub async fn start(&self) {
        info_meta("Cluster controller started successfully");
    }

    pub async fn stop(&self) {
        info_meta("Cluster controller stop");
    }
}
