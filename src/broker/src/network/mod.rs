use rector::Rector;

use self::rector::ConnectionManager;
mod network;
mod rector;

pub struct NetworkServer{

}

impl NetworkServer {
    pub fn start(){
        // let rector = Rector::new();
        let connection_manager = ConnectionManager::new(1000);
    }
}
