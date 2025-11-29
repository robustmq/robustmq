use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum PushModel {
    QuickFailure,
    RetryFailure,
}

pub fn get_push_model(client_id: &str, topic_name: &str) -> PushModel {
    if let Some(model) = get_push_model_by_client_id(client_id) {
        return model;
    }

    if let Some(model) = get_push_model_by_topic_name(topic_name) {
        return model;
    }
    PushModel::QuickFailure
}

fn get_push_model_by_topic_name(_topic_name: &str) -> Option<PushModel> {
    Some(PushModel::QuickFailure)
}

fn get_push_model_by_client_id(_client_id: &str) -> Option<PushModel> {
    None
}
