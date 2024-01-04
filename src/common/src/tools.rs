use std::{fs, path::Path};

pub fn create_fold(fold: String) {
    if !Path::new(&fold).exists() {
        fs::create_dir_all(fold).unwrap();
    }
}
