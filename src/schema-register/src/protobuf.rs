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

pub fn protobuf_validate() -> Result<bool, CommonError> {
    Ok(true)
}

#[cfg(test)]
mod test {
    use crate::protobuf::protobuf_validate;
    // use protobuf::descriptor::FileDescriptorProto;
    // use protobuf::reflect::FileDescriptor;
    // use protobuf::text_format::parse_from_str;
    use bytes::Bytes;
    use protofish::decode::UnknownValue;
    use protofish::prelude::*;

    #[test]
    pub fn protobuf_validate_test() {
        let result = protobuf_validate();
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    pub fn protobuf_test() {
        // 动态提供的 Protobuf 定义字符串
        // let proto_content = r#"
        //     syntax = "proto3";
        //     package mypackage;

        //     message Person {
        //         string name = 1;
        //         uint32 age = 2;
        //     }
        // "#;

        // // 将 Protobuf 定义字符串解析为 FileDescriptorProto
        // let file_descriptor_proto = parse_from_str::<FileDescriptorProto>(proto_content).unwrap();
        // println!("{:?}", file_descriptor_proto);
        // // // 创建 FileDescriptor
        // let file_descriptor = FileDescriptor::new_dynamic(file_descriptor_proto, None).unwrap();

        // // 获取消息类型描述符
        // let message_descriptor = file_descriptor
        //     .message_by_full_name("mypackage.Person")
        //     .unwrap();

        // // 待校验的二进制数据
        // let data = vec![...]; // 替换为你的二进制数据

        // // 尝试将数据反序列化为 DynamicMessage
        // let dynamic_message = DynamicMessage::decode(message_descriptor, &data)?;

        // println!("数据符合 Protobuf 定义");
        // println!("内容: {:?}", dynamic_message);
    }

    #[test]
    pub fn protofish_example_test() {
        let context = Context::parse(&[r#"
            syntax = "proto3";
            package Proto;

            message Request { string kind = 1; }
            message Response { int32 distance = 1; }
            service Fish {
                rpc Swim( Request ) returns ( Response );
            }
        "#])
        .unwrap();

        let service = context.get_service("Proto.Fish").unwrap();
        let rpc = service.rpc_by_name("Swim").unwrap();

        let input = context.decode(rpc.input.message, b"\x0a\x05Perch");
        assert_eq!(input.fields[0].number, 1);
        assert_eq!(input.fields[0].value, Value::String(String::from("Perch")));

        let output = context.decode(rpc.output.message, b"\x08\xa9\x46");
        assert_eq!(output.fields[0].number, 1);
        assert_eq!(output.fields[0].value, Value::Int32(9001));

        let bytes = b"\x12\x07Unknown\x0a\x0fAtlantic ";
        let request = context.get_message("Proto.Request").unwrap();
        let value = request.decode(bytes, &context);
        assert_eq!(value.fields[0].number, 2);
        assert_eq!(
            value.fields[0].value,
            Value::Unknown(UnknownValue::VariableLength(Bytes::from_static(b"Unknown")))
        );
        assert_eq!(
            value.fields[1].value,
            Value::Incomplete(2, Bytes::from_static(b"\x0fAtlantic "))
        );

        let encoded = value.encode(&context);
        assert_eq!(encoded, &bytes[..]);
    }

    #[test]
    pub fn protofish_schema_test() {
        let context = Context::parse(&[r#"
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
        "#])
        .unwrap();

        // ----- Experience -----
        // Experience {
        //     company: "Google",
        //     position: "Software Engineer",
        //     start_year: 2021,
        //     end_year: 2023,
        //     skills: ["Java", "Google Cloud", "Software Testing"]
        // }
        let experience_raw = b"\x0a\x06Google\x12\x11Software Engineer\x18\xe5\x0f\x20\xe7\x0f\x2a\x04Java\x2a\x0cGoogle Cloud\x2a\x10Software Testing";

        let experience_message = context.get_message("MyPackage.Experience").unwrap();
        let experience_value = experience_message.decode(experience_raw, &context);

        // company
        assert_eq!(experience_value.fields[0].number, 1);
        assert_eq!(
            experience_value.fields[0].value,
            Value::String(String::from("Google"))
        );

        // position
        assert_eq!(experience_value.fields[1].number, 2);
        assert_eq!(
            experience_value.fields[1].value,
            Value::String(String::from("Software Engineer"))
        );

        // start_year
        assert_eq!(experience_value.fields[2].number, 3);
        assert_eq!(experience_value.fields[2].value, Value::UInt32(2021));

        // end_year
        assert_eq!(experience_value.fields[3].number, 4);
        assert_eq!(experience_value.fields[3].value, Value::UInt32(2023));

        // skills, currently protofish does not support repeated strings
        assert_eq!(experience_value.fields[4].number, 5);
        assert_eq!(
            experience_value.fields[4].value,
            Value::String(String::from("Java"))
        );

        assert_eq!(experience_value.fields[5].number, 5);
        assert_eq!(
            experience_value.fields[5].value,
            Value::String(String::from("Google Cloud"))
        );

        assert_eq!(experience_value.fields[6].number, 5);
        assert_eq!(
            experience_value.fields[6].value,
            Value::String(String::from("Software Testing"))
        );

        // ----- Person -----
        // Person {
        //     name: "John",
        //     age: 30,
        //     experience: [experience]
        // }
        let person_raw = b"\x0a\x04John\x10\x1e\x1a\x47\x0a\x06Google\x12\x11Software Engineer\x18\xe5\x0f\x20\xe7\x0f\x2a\x04Java\x2a\x0cGoogle Cloud\x2a\x10Software Testing";

        let person_message = context.get_message("MyPackage.Person").unwrap();
        let person_value = person_message.decode(person_raw, &context);

        // name
        assert_eq!(person_value.fields[0].number, 1);
        assert_eq!(
            person_value.fields[0].value,
            Value::String(String::from("John"))
        );

        // age
        assert_eq!(person_value.fields[1].number, 2);
        assert_eq!(person_value.fields[1].value, Value::UInt32(30));

        // experience
        assert_eq!(person_value.fields[2].number, 3);
        assert_eq!(
            person_value.fields[2].value,
            Value::Message(Box::new(experience_value))
        );
    }
}
