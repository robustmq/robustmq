pub struct PlacementCliCommandParam {
    pub server: String,
    pub action: String,
}

pub enum PlacementActionType {
    STATUS,
}

impl From<String> for PlacementActionType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "status" => PlacementActionType::STATUS,
            _ => panic!("Invalid action type {}", s),
        }
    }
}

pub struct PlacementCenterCommand {}

impl PlacementCenterCommand {
    pub fn new() -> Self {
        return PlacementCenterCommand {};
    }
    pub async fn start(&self, params: PlacementCliCommandParam) {
        let action_type = PlacementActionType::from(params.action.clone());
        match action_type {
            PlacementActionType::STATUS => {
                println!("placement status");
            }
        }
    }
}
