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

use common_base::error::common::CommonError;
use valico::json_schema::{self};

pub fn json_validate(json_schema: &str, data: &str) -> Result<bool, CommonError> {
    let schema = serde_json::from_str(json_schema)?;

    let mut scope = json_schema::Scope::new();
    let schema = scope.compile_and_return(schema, false)?;

    let json_data = serde_json::from_str(data)?;
    let state = schema.validate(&json_data);
    Ok(state.is_valid())
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use valico::json_schema;

    use crate::json::json_validate;

    #[test]
    pub fn json_validate_test() {
        let schema = r#"{
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer", "minimum": 0 }
            },
            "required": ["name"]
        }"#;

        let data = r#"{
            "name": "John Doe",
            "age": 30
        }"#;

        let result = json_validate(schema, data);
        println!("{result:?}");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    pub fn read_offset_data_test() {
        let json_schema = r#"{
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer", "minimum": 0 }
            },
            "required": ["name"]
        }"#;

        // Defining JSON Schema
        let schema = serde_json::from_str(json_schema).unwrap();

        // Create the Scope and compile the Schema
        let mut scope = json_schema::Scope::new();
        let schema = scope.compile_and_return(schema, false).unwrap();

        // JSON data to validate
        let data = json!({
            "name": "John Doe",
            "age": 30
        });

        // Validate data
        let state = schema.validate(&data);
        if state.is_valid() {
            println!("JSON is valid!");
        } else {
            println!("JSON is invalid: {:?}", state.errors);
        }
    }
}
