use super::{
  calc_total_length, extract_fixed_data, parse_length, RespDecode, RespEncode, RespError,
  RespFrame, BUF_CAP, CRLF_LEN,
};
use anyhow::Result;
use bytes::{Buf, BytesMut};
use std::ops::Deref;

const NULL_ARRAY: &str = "*-1\r\n";

#[derive(Debug, PartialEq, PartialOrd, Clone, Default)]
pub struct RespArray(pub(crate) Option<Vec<RespFrame>>);

impl RespArray {
  pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
    RespArray(Some(s.into()))
  }
}

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
//    - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
impl RespEncode for RespArray {
  fn encode(self) -> Vec<u8> {
    match self.0 {
      Some(data) => {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(format!("*{}\r\n", data.len()).as_bytes());
        for frame in data {
          buf.extend_from_slice(&frame.encode());
        }
        buf
      }
      None => NULL_ARRAY.as_bytes().to_vec(),
    }
  }
}

// FIXME: need to handle incomplete
impl RespDecode for RespArray {
  const PREFIX: &'static str = "*";
  fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
    let is_null = extract_fixed_data(buf, NULL_ARRAY, "NullArray").is_ok();
    if is_null {
      return Ok(RespArray::default());
    }

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

    Ok(RespArray(Some(frames)))
  }
  fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
    let (end, len) = parse_length(buf, Self::PREFIX)?;
    calc_total_length(buf, end, len, Self::PREFIX)
  }
}

impl Deref for RespArray {
  type Target = Option<Vec<RespFrame>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{BulkString, RespFrame};

  #[test]
  fn test_array_encode() {
    let frame: RespFrame = RespArray::new(vec![
      BulkString::new(b"get".to_vec()).into(),
      BulkString::new(b"hello".to_vec()).into(),
    ])
    .into();
    assert_eq!(frame.encode(), b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");
  }

  #[test]
  fn test_array_decode() -> Result<()> {
    let mut buf = BytesMut::new();
    buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");

    let frame = RespArray::decode(&mut buf)?;
    assert_eq!(frame, RespArray::new([b"get".into(), b"hello".into()]));

    buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n");
    let ret = RespArray::decode(&mut buf);
    assert_eq!(ret.unwrap_err(), RespError::NotComplete);

    buf.extend_from_slice(b"$5\r\nhello\r\n");
    let frame = RespArray::decode(&mut buf)?;
    assert_eq!(frame, RespArray::new([b"get".into(), b"hello".into()]));

    Ok(())
  }

  #[test]
  fn test_null_array_encode() {
    let frame: RespFrame = RespArray::default().into();
    assert_eq!(frame.encode(), b"*-1\r\n");
  }

  #[test]
  fn test_null_array_decode() -> Result<()> {
    let mut buf = BytesMut::new();
    buf.extend_from_slice(b"*-1\r\n");

    let frame = RespArray::decode(&mut buf)?;
    assert_eq!(frame, RespArray::default());

    Ok(())
  }
}
