/*
- 如何解析 Frame
    - simple string: "+OK\r\n"
    - error: "-Error message\r\n"
    - bulk error: "!<length>\r\n<error>\r\n"
    - integer: ":[<+|->]<value>\r\n"
    - bulk string: "$<length>\r\n<data>\r\n"
    - null bulk string: "$-1\r\n"
    - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
        - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
    - null array: "*-1\r\n"
    - null: "_\r\n"
    - boolean: "#<t|f>\r\n"
    - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
    - map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
    - set: "~<number-of-elements>\r\n<element-1>...<element-n>"
 */
use crate::{BulkString, RespArray, SimpleString};

use super::{RespEncode, RespNull};

const BUF_CAP: usize = 4096;

// - simple string: "+OK\r\n"
impl RespEncode for SimpleString {
  fn encode(self) -> Vec<u8> {
    format!("+{}\r\n", self.0).into_bytes()
  }
}

// - null: "_\r\n"
impl RespEncode for RespNull {
  fn encode(self) -> Vec<u8> {
    b"_\r\n".to_vec()
  }
}

// - bulk string: "$<length>\r\n<data>\r\n"
impl RespEncode for BulkString {
  fn encode(self) -> Vec<u8> {
    let mut buf = Vec::with_capacity(self.len() + 16);
    buf.extend_from_slice(format!("${}\r\n", self.len()).as_bytes());
    buf.extend_from_slice(&self);
    buf.extend_from_slice(b"\r\n");
    buf
  }
}
// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
//    - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
impl RespEncode for RespArray {
  fn encode(self) -> Vec<u8> {
    let mut buf = Vec::with_capacity(BUF_CAP);
    buf.extend_from_slice(format!("*{}\r\n", self.0.len()).as_bytes());
    for frame in self.0 {
      buf.extend_from_slice(&frame.encode());
    }
    buf
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::RespFrame;

  #[test]
  fn test_simple_string_encode() {
    let frame: RespFrame = SimpleString::new("OK".to_string()).into();
    assert_eq!(frame.encode(), b"+OK\r\n");
  }
}
