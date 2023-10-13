use log::{info, error, warn};
use log4rs;
fn main() {
    println!("Hello, world! RobustMQ");
    log4rs::init_file("log4rs.yml",Default::default()).unwrap();
    info!("info booting up") ;
    error!("error goes to stderr and file");
    warn!("warn goes to stderr and file");
}