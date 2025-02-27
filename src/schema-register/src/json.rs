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
use serde_json::{json, Value};
use valico::json_schema::{self};

pub fn json_validate(schema: &str, data: Value) -> Result<bool, CommonError> {
    let schema = json!(schema);

    let mut scope = json_schema::Scope::new();
    let schema = scope.compile_and_return(schema, false)?;

    let state = schema.validate(&data);
    Ok(state.is_valid())
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use valico::json_schema;

    #[test]
    pub fn json_validate_test() {}

    #[test]
    pub fn read_offset_data_test() {
        // Defining JSON Schema
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer", "minimum": 0 }
            },
            "required": ["name"]
        });

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
