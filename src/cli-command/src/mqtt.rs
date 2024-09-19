pub struct MqttCliCommandParam {
    pub server: String,
    pub action: String,
}

pub enum MqttActionType {
    STATUS,
}

impl From<String> for MqttActionType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "status" => MqttActionType::STATUS,
            _ => panic!("Invalid action type {}", s),
        }
    }
}

pub struct MqttBrokerCommand {}

impl MqttBrokerCommand {
    pub fn new() -> Self {
        return MqttBrokerCommand {};
    }

    pub async fn start(&self, params: MqttCliCommandParam) {
        let action_type = MqttActionType::from(params.action.clone());
        match action_type {
            MqttActionType::STATUS => {
                println!("mqtt status");
            }
        }
    }
}
