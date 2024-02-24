use std::sync::Arc;

use crate::broker_cluster::BrokerCluster;

#[derive(Default,Clone)]
pub struct BrokerServerController {
    pub storage_cluser: Arc<BrokerCluster>,
}


impl BrokerServerController {
    pub fn new(storage_cluser: Arc<BrokerCluster>) -> BrokerServerController{
        let mut bsc = BrokerServerController::default();
        bsc.storage_cluser = storage_cluser;
        return bsc;
    }

    pub async fn start(&self){
        
    }
}