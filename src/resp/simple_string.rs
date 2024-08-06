use super::{extract_simple_frame_data, RespDecode, RespEncode, RespError, CRLF_LEN};
use anyhow::Result;
use bytes::BytesMut;
use std::ops::Deref;

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct SimpleString(pub String);

impl SimpleString {
  pub fn new(s: impl Into<String>) -> Self {
    SimpleString(s.into())
  }
}

// - simple string: "+OK\r\n"
impl RespEncode for SimpleString {
  fn encode(self) -> Vec<u8> {
    format!("+{}\r\n", self.0).into_bytes()
  }
}

impl RespDecode for SimpleString {
  const PREFIX: &'static str = "+";
  fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
    let end = extract_simple_frame_data(buf, Self::PREFIX)?;
    let data = buf.split_to(end + CRLF_LEN);
    let s = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]);
    Ok(SimpleString::new(s))
  }
  fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
    let end = extract_simple_frame_data(buf, Self::PREFIX)?;
    Ok(end + CRLF_LEN)
  }
}

impl Deref for SimpleString {
  type Target = String;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::RespFrame;
  use bytes::BufMut;

  #[test]
  fn test_simple_string_encode() {
    let frame: RespFrame = SimpleString::new("OK".to_string()).into();
    assert_eq!(frame.encode(), b"+OK\r\n");
  }

  #[test]
  fn test_simple_string_decode() -> Result<()> {
    let mut buf = BytesMut::new();
    buf.extend_from_slice(b"+OK\r\n");

    let frame = SimpleString::decode(&mut buf)?;
    assert_eq!(frame, SimpleString::new("OK".to_string()));

    buf.extend_from_slice(b"+hello\r");

    let frame = SimpleString::decode(&mut buf);
    assert_eq!(frame.unwrap_err(), RespError::NotComplete);

    buf.put_u8(b'\n');
    let frame = SimpleString::decode(&mut buf)?;
    assert_eq!(frame, SimpleString::new("hello".to_string()));

    Ok(())
  }
}
