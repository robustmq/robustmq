use super::security::Authentication;

pub struct PlaintextData {
    pub username: String,
    pub password: String,
}
pub struct Plaintext {}

impl Authentication for Plaintext {
    fn apply() {}
}
