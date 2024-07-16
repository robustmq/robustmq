/*
 * Copyright (c) 2023 robustmq team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use super::*;

impl Disconnect {
    fn mqttv4() -> Disconnect {
        return Disconnect {
            reason_code: Some(DisconnectReasonCode::NormalDisconnection),
        };
    }
}

// In MQTT V4(3.1.1) there are no reason code and properties for disconnect packet
pub fn write(_disconnect: &Disconnect, payload: &mut BytesMut) -> Result<usize, Error> {
    payload.put_slice(&[0xE0, 0x00]); // 1st byte of fixed header in disconnect packet is 0x1110 0000
                                      // 1110(14) is the packet type of disconnect
                                      // 0000 is reserved connection flags which must be 0
                                      // otherwise the server must be shutdown the connection
    Ok(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disconnect() {
        let mut buffer = BytesMut::new();
        let disconnect = Disconnect::mqttv4();
        // test write function
        write(&disconnect, &mut buffer);
        assert_eq!(buffer.get_u8(), 0b11100000);
    }
}
