use rlog;
use serde::Deserialize;
use std::fs;
use std::path;
use toml;

pub const DEFAULT_SERVER_CONFIG: &str = "config/server.toml";

#[derive(Debug, Deserialize)]
pub struct RobustServerConfig {
    pub addr: String,
    pub broker: Broker,
    pub admin: Admin,
    pub prometheus: Prometheus,
}

#[derive(Debug, Deserialize)]
pub struct Broker {
    pub port: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct Admin {
    pub port: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct Prometheus {
    pub port: Option<u16>,
}

pub fn new(config_path: &String) -> RobustServerConfig {
    rlog::info(&format!("Configuration file path:{}.", config_path));

    if !path::Path::new(config_path).exists() {
        panic!("The configuration file does not exist.");
    }

    let content: String = fs::read_to_string(&config_path).expect(&format!(
        "Failed to read the configuration file. File path:{}.",
        config_path
    ));

    rlog::info(&format!(
        "server config content:\n============================\n{}\n============================\n",
        content
    ));

    let server_config: RobustServerConfig = toml::from_str(&content).unwrap();
    return server_config;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        log4rs::init_file(
            format!("../../{}", rlog::DEFAULT_LOG_CONFIG),
            Default::default(),
        )
        .unwrap();

        let conf: RobustServerConfig = parse(&format!("../../{}", DEFAULT_SERVER_CONFIG));

        assert_eq!(conf.addr, "127.0.0.1".to_string());
        assert_eq!(conf.broker.port, Some(1226));
        assert_eq!(conf.admin.port, Some(1227));
        assert_eq!(conf.prometheus.port, Some(9184));
    }
}
