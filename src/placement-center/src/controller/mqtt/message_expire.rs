pub struct MessageExpire {}

impl MessageExpire {
    pub fn new() -> Self {
        return MessageExpire {};
    }

    pub async fn retain_message_expire(&self) {}

    pub async fn last_will_message_expire(&self) {}
}
