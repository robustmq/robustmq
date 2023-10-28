use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RobustConfig {
    pub addr: String,
    pub broker: Broker,
    pub admin: Admin
}

#[derive(Debug, Deserialize)]
pub struct Broker {
    pub port: Option<u16>,
    pub work_thread: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct Admin {
    pub port: Option<u16>,
    pub work_thread: Option<u16>,
}