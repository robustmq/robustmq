pub mod logo;
use log::error;
use logo::DEFAULT_PLACEMENT_CENTER_CONFIG;

use crate::tools::read_file;

pub fn version() -> String {
    let content = match read_file(&DEFAULT_PLACEMENT_CENTER_CONFIG.to_string()) {
        Ok(data) => data,
        Err(e) => {
            error!("{}", e.to_string());
            return "-".to_string();
        }
    };
    return content;
}
