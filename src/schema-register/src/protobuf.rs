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
}
