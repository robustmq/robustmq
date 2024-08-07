

#[derive(Debug, PartialEq, Eq)]
pub struct TAuthUser {
    pub id: u64,
    pub username: String,
    pub password: String,
    pub salt: String,
    pub is_superuser: String,
    pub created: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TAuthAcl {
    pub id: u64,
    pub allow: u64,
    pub ipaddr: String,
    pub username: String,
    pub clientid: String,
    pub access: u64,
    pub topic: String,
}
