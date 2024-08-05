use common_base::errors::RobustMQError;
use mysql::Pool;

pub fn build_mysql_conn_pool(addr: &str) -> Result<Pool, RobustMQError> {
    match Pool::new(addr) {
        Ok(pool) => return Ok(pool),
        Err(e) => {
            return Err(RobustMQError::CommmonError(e.to_string()));
        }
    }
}
