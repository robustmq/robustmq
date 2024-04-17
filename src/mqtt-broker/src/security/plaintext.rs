use bytes::Bytes;
use protocol::mqtt::Login;
use super::security::Authentication;

pub struct Plaintext {}

impl Authentication for Plaintext {
    fn apply(login: Option<Login>, _: Option<Bytes>) {

    }
}
