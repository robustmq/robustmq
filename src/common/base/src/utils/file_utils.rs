// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::env::temp_dir;
use std::fs::read_dir;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::{env, io};

use crate::tools::unique_id;

/// Get the project root (relative to closest Cargo.lock file)
pub fn get_project_root() -> io::Result<PathBuf> {
    let path = env::current_dir()?;
    let path_ancestors = path.as_path().ancestors();

    for p in path_ancestors {
        let has_cargo = read_dir(p)?.any(|p| p.unwrap().file_name() == "Cargo.lock");
        if has_cargo {
            return Ok(PathBuf::from(p));
        }
    }

    Err(io::Error::new(
        ErrorKind::NotFound,
        "Ran out of places to find Cargo.toml",
    ))
}

pub fn test_temp_dir() -> String {
    format!("{}{}/", temp_dir().to_str().unwrap(), unique_id())
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use crate::utils::file_utils::get_project_root;

    use super::test_temp_dir;

    #[test]
    fn it_should_find_our_project_root() {
        let crate_name = "[workspace]";

        let project_root = get_project_root().expect("There is no project root");

        let toml_path = project_root.to_str().unwrap().to_owned() + "/Cargo.toml";
        let toml_string = read_to_string(toml_path).unwrap();

        assert!(toml_string.contains(crate_name));
    }

    #[test]
    fn test_temp_dir_test() {
        let path = test_temp_dir();
        println!("{}", path);
        let path = test_temp_dir();
        println!("{}", path);
    }
}
