use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password: String,
    pub salt: u16,
    pub is_superuser: bool,
    pub create_time: u128,
}
