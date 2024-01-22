use std::{fs, path::Path};

pub fn create_fold(fold: String) {
    if !Path::new(&fold).exists() {
        fs::create_dir_all(fold).unwrap();
    }
}

pub fn handle_running(threads: Vec<Result<std::thread::JoinHandle<()>, std::io::Error>>) {
    for th in threads {
        th.unwrap().join().unwrap();
    }
}
