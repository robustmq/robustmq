pub const DEFAULT_LOG_CONFIG: &str = "config/log4rs.yml";

pub fn info(msg: &str) -> () {
    log::info!(target:"app::server", "{}",msg)
}

pub fn server_info(msg: &str) -> () {
    log::info!(target:"app::server", "{}",msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_print() {
        log4rs::init_file(format!("../../{}", DEFAULT_LOG_CONFIG), Default::default()).unwrap();
        info("lobo");
        server_info("server lobo");
    }
}
