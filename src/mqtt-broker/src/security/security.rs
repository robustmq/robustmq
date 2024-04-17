use bytes::Bytes;
use protocol::mqtt::Login;

pub trait Authentication {
    fn apply(login: Option<Login>, authentication_data: Option<Bytes>);
}
