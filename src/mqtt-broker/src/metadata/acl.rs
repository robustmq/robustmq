pub struct Acl {
    pub allow: AclAllow,
    pub ip_addr: String,
    pub username: String,
    pub client_id: String,
    pub access: AclAccess,
    pub topic: String,
}

pub enum AclAllow {
    Deny,
    Allow,
}

pub enum AclAccess {
    Subscribe,
    Publish,
    PubSub,
}
