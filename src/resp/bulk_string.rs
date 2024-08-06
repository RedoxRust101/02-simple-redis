use super::{parse_length, RespDecode, RespEncode, RespError, CRLF_LEN};
use anyhow::Result;
use bytes::{Buf, BytesMut};
use std::ops::Deref;

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct BulkString(pub(crate) Vec<u8>);

impl BulkString {
  pub fn new(s: impl Into<Vec<u8>>) -> Self {
    BulkString(s.into())
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

impl<const N: usize> From<&[u8; N]> for BulkString {
  fn from(s: &[u8; N]) -> Self {
    BulkString(s.to_vec())
  }
}

impl From<String> for BulkString {
  fn from(s: String) -> Self {
    BulkString(s.into_bytes())
  }
}

impl From<&str> for BulkString {
  fn from(s: &str) -> Self {
    BulkString(s.as_bytes().to_vec())
  }
}

impl Deref for BulkString {
  type Target = [u8];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::RespFrame;

  #[test]
  fn test_bulk_string_encode() {
    let frame: RespFrame = BulkString::new(b"hello".to_vec()).into();
    assert_eq!(frame.encode(), b"$5\r\nhello\r\n");
  }

  #[test]
  fn test_bulk_string_decode() -> Result<()> {
    let mut buf = BytesMut::new();
    buf.extend_from_slice(b"$5\r\nhello\r\n");

    let frame = BulkString::decode(&mut buf)?;
    assert_eq!(frame, BulkString::new(b"hello"));

    buf.extend_from_slice(b"$5\r\nworld");

    let ret = BulkString::decode(&mut buf);
    assert_eq!(ret.unwrap_err(), RespError::NotComplete);

    buf.extend_from_slice(b"\r\n");

    let frame = BulkString::decode(&mut buf)?;
    assert_eq!(frame, BulkString::new(b"world"));

    Ok(())
  }
}
