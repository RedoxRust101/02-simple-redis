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
use bytes::{Buf, BytesMut};

use super::{BulkString, RespArray};

const CRLF: &[u8] = b"\r\n";
const CRLF_LEN: usize = CRLF.len();

impl RespDecode for RespFrame {
  const PREFIX: &'static str = "";
  fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
    let mut iter = buf.iter().peekable();
    match iter.peek() {
      Some(b'+') => {
        let frame = SimpleString::decode(buf)?;
        Ok(frame.into())
      }
      Some(b'$') => {
        let frame = BulkString::decode(buf)?;
        Ok(frame.into())
      }
      _ => Err(RespError::InvalidFrameType(format!(
        "{:?} from RespFrame decode()",
        String::from_utf8_lossy(buf)
      ))),
    }
  }
  fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
    let mut iter = buf.iter().peekable();
    match iter.peek() {
      Some(b'+') => SimpleString::expect_length(buf),
      Some(b'$') => BulkString::expect_length(buf),
      _ => Err(RespError::InvalidFrameType(format!(
        "{:?} from RespFrame expect_length()",
        String::from_utf8_lossy(buf)
      ))),
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

impl RespDecode for BulkString {
  const PREFIX: &'static str = "$";
  fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
    let (end, len) = parse_length(buf, Self::PREFIX)?;
    let remained = &buf[end + CRLF_LEN..];
    if remained.len() < len + CRLF_LEN {
      return Err(RespError::NotComplete);
    }
    buf.advance(end + CRLF_LEN);

    let data = buf.split_to(len + CRLF_LEN);
    Ok(BulkString::new(data[..len].to_vec()))
  }
  fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
    let (end, len) = parse_length(buf, Self::PREFIX)?;
    Ok(end + CRLF_LEN + len + CRLF_LEN)
  }
}

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
// - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
// FIXME: need to handle incomplete
impl RespDecode for RespArray {
  const PREFIX: &'static str = "*";
  fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
    let (end, len) = parse_length(buf, Self::PREFIX)?;
    let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;

    if buf.len() < total_len {
      return Err(RespError::NotComplete);
    }

    buf.advance(end + CRLF_LEN);

    let mut frames = Vec::with_capacity(len);
    for _ in 0..len {
      let frame = RespFrame::decode(buf)?;
      frames.push(frame);
    }

    Ok(RespArray(frames))
  }
  fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
    let (end, len) = parse_length(buf, Self::PREFIX)?;
    calc_total_length(buf, end, len, Self::PREFIX)
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

fn parse_length(buf: &[u8], prefix: &str) -> Result<(usize, usize), RespError> {
  let end = extract_simple_frame_data(buf, prefix)?;
  let s = String::from_utf8_lossy(&buf[prefix.len()..end]);
  Ok((end, s.parse()?))
}

fn calc_total_length(buf: &[u8], end: usize, len: usize, prefix: &str) -> Result<usize, RespError> {
  let mut total = end + CRLF_LEN;
  let mut data = &buf[total..];
  match prefix {
    "*" => {
      // find nth CRLF in the buffer, for array and set, we need to find 1 CRLF for each element
      for _ in 0..len {
        let frame_len = RespFrame::expect_length(data)?;
        data = &data[frame_len..];
        total += frame_len;
      }
      Ok(total)
    }
    _ => Ok(len + CRLF_LEN),
  }
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
