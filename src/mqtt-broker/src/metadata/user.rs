use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password: String,
    pub salt: UserSalt,
    pub is_superuser: bool,
    pub create_time: u128,
}


#[derive(Clone, Serialize, Deserialize)]
pub enum UserSalt {
    Md5,
    Sha256,
}
