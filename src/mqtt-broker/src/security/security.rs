use bytes::Bytes;
use common_base::errors::RobustMQError;
use protocol::mqtt::Login;

pub trait Authentication {
    fn apply(
        login: Option<Login>,
        authentication_data: Option<Bytes>,
    ) -> Result<bool, RobustMQError>;
}

pub fn authentication_login(
    login: Option<Login>,
    authentication_method: Option<String>,
    authentication_data: Option<Bytes>,
) -> Result<bool, RobustMQError> {
    
    if let Some(info) = login {
        return Ok(false);
    }

    return Ok(true);
}
