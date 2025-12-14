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
use metadata_struct::schema::SchemaData;
use protofish::{decode::Value, prelude::Context};

pub fn protobuf_validate(
    schema_data: &SchemaData,
    data: &[u8],
    message_name: &str,
) -> Result<bool, CommonError> {
    let context = Context::parse([schema_data.schema.as_str()]).map_err(|err| {
        CommonError::CommonError(format!(
            "Failed to parse schema {}: {}",
            schema_data.name.as_str(),
            err
        ))
    })?;

    let message = context.get_message(message_name).ok_or_else(|| {
        CommonError::CommonError(format!(
            "Message {} not found in schema {}",
            message_name,
            schema_data.name.as_str()
        ))
    })?;

    let decoded = message.decode(data, &context);

    // Check if there are any unknown or incomplete fields
    for field in decoded.fields {
        match &field.value {
            Value::Unknown(_) | Value::Incomplete(_, _) => {
                return Ok(false); // Invalid if unknown or incomplete fields exist
            }
            _ => {} // Otherwise, it's fine
        }
    }

    Ok(true)
}

#[cfg(test)]
mod test {
    use crate::protobuf::protobuf_validate;
    use metadata_struct::schema::{SchemaData, SchemaType};

    #[test]
    pub fn protobuf_validate_test() {
        protofish_example_test();
        protofish_schema_test();
    }

    pub fn protofish_example_test() {
        let schema = r#"
            syntax = "proto3";
            package Proto;

            message Request { string kind = 1; }
            message Response { int32 distance = 1; }
            service Fish {
                rpc Swim( Request ) returns ( Response );
            }
        "#;

        let schema_data = SchemaData {
            name: "Proto".to_string(),
            schema_type: SchemaType::PROTOBUF,
            desc: "".to_string(),
            schema: schema.to_string(),
        };

        let res = protobuf_validate(&schema_data, b"\x0a\x05Perch", "Proto.Request");
        assert!(res.is_ok());
        assert!(res.unwrap());

        let res = protobuf_validate(&schema_data, b"\x08\xa9\x46", "Proto.Response");
        assert!(res.is_ok());
        assert!(res.unwrap());

        let res = protobuf_validate(
            &schema_data,
            b"\x12\x07Unknown\x0a\x0fAtlantic ",
            "Proto.Request",
        );
        assert!(res.is_ok());
        assert!(!res.unwrap());
    }

    pub fn protofish_schema_test() {
        let schema = r#"
            syntax = "proto3";
            package MyPackage;

            message Experience {
                string company = 1;
                string position = 2;
                uint32 start_year = 3;
                uint32 end_year = 4;
                repeated string skills = 5;
            }

            message Person {
                string name = 1;
                uint32 age = 2;
                repeated Experience experience = 3;
            }
        "#;

        let schema_data = SchemaData {
            name: "MyPackage".to_string(),
            schema_type: SchemaType::PROTOBUF,
            desc: "".to_string(),
            schema: schema.to_string(),
        };

        // ----- Experience -----
        // Experience {
        //     company: "Google",
        //     position: "Software Engineer",
        //     start_year: 2021,
        //     end_year: 2023,
        //     skills: ["Java", "Google Cloud", "Software Testing"]
        // }
        let res = protobuf_validate(
            &schema_data,
            b"\x0a\x06Google\x12\x11Software Engineer\x18\xe5\x0f\x20\xe7\x0f\x2a\x04Java\x2a\x0cGoogle Cloud\x2a\x10Software Testing",
            "MyPackage.Experience"
        );

        assert!(res.is_ok());
        assert!(res.unwrap());

        // ----- Person -----
        // Person {
        //     name: "John",
        //     age: 30,
        //     experience: [experience]
        // }
        let res = protobuf_validate(
            &schema_data,
            b"\x0a\x04John\x10\x1e\x1a\x47\x0a\x06Google\x12\x11Software Engineer\x18\xe5\x0f\x20\xe7\x0f\x2a\x04Java\x2a\x0cGoogle Cloud\x2a\x10Software Testing",
            "MyPackage.Person"
        );

        assert!(res.is_ok());
        assert!(res.unwrap());
    }
}
