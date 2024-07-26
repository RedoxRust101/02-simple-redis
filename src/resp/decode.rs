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

use crate::{RespDecode, RespError, RespFrame, SimpleString};
use bytes::BytesMut;

const CRLF: &[u8] = b"\r\n";
const CRLF_LEN: usize = CRLF.len();

impl RespDecode for RespFrame {
  const PREFIX: &'static str = "";
  fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
    let mut iter = buf.iter().peekable();
    match iter.peek() {
      Some(&b'+') => {
        let frame = SimpleString::decode(buf)?;
        Ok(frame.into())
      }
      _ => Err(RespError::InvalidFrameType(format!("expect_length: unknown type: {:?}", buf))),
    }
  }
  fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
    let mut iter = buf.iter().peekable();
    match iter.peek() {
      Some(&b'+') => SimpleString::expect_length(buf),
      _ => Err(RespError::NotComplete),
    }
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

fn extract_simple_frame_data(buf: &[u8], prefix: &str) -> Result<usize, RespError> {
  if buf.len() < 3 {
    return Err(RespError::NotComplete);
  }

  if !buf.starts_with(prefix.as_bytes()) {
    return Err(RespError::InvalidFrameType(format!(
      "expect SimpleString({}), got: {:?}",
      prefix, buf
    )));
  }

  let end = find_crlf(buf, 1).ok_or(RespError::NotComplete)?;

  Ok(end)
}

fn find_crlf(buf: &[u8], nth: usize) -> Option<usize> {
  let mut count = 0;
  for i in 1..buf.len() - 1 {
    if buf[i] == b'\r' && buf[i + 1] == b'\n' {
      count += 1;
      if count == nth {
        return Some(i);
      }
    }
  }

  None
}

#[cfg(test)]
mod tests {
  use super::*;
  use anyhow::Result;
  use bytes::BytesMut;

  #[test]
  fn test_simple_string_decode() -> Result<()> {
    let mut buf = BytesMut::new();
    buf.extend_from_slice(b"+OK\r\n");

    let frame = SimpleString::decode(&mut buf)?;
    assert_eq!(frame, SimpleString::new("OK".to_string()));

    buf.extend_from_slice(b"+hello\r");

    let frame = SimpleString::decode(&mut buf);
    assert_eq!(frame.unwrap_err(), RespError::NotComplete);

    // TODO: no methed named put_u8
    buf.extend_from_slice(b"\n");
    let frame = SimpleString::decode(&mut buf)?;
    assert_eq!(frame, SimpleString::new("hello".to_string()));

    Ok(())
  }
}
