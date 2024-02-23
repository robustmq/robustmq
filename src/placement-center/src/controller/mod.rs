pub mod storage_controller;
pub mod broker_controller;
pub mod replica_manage;
use common::log::info_meta;

use self::storage_controller::StorageEngineController;

pub struct Controller {}

impl Controller {
    pub fn new() -> Self {
        Controller {}
    }

    pub async fn start(&self) {
        // start storage engin controller
        let se_ctrl = StorageEngineController::new();
        se_ctrl.start();

        // start broker server controller

        info_meta("Cluster controller started successfully");
    }

    pub async fn stop(&self) {
        info_meta("Cluster controller stop");
    }
}
