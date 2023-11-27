use rector::Rector;

use self::connection::ConnectionManager;
mod connection;
mod network;
mod rector;

pub struct NetworkServer {}

impl NetworkServer {
    pub fn start() {
        // let rector = Rector::new();
        let connection_manager = ConnectionManager::new(1000);
    }
}

#[cfg(test)]
mod test {
    #[test]

    fn network_server_test() {
        
    }
}
