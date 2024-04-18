use super::security::Authentication;
use bytes::Bytes;
use common_base::errors::RobustMQError;
use protocol::mqtt::Login;

pub struct Plaintext {}

impl Authentication for Plaintext {
    fn apply(login: Option<Login>, _: Option<Bytes>) -> Result<bool, RobustMQError> {
        return Ok(false);
    }
}
