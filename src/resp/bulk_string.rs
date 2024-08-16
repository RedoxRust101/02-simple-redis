use super::{extract_fixed_data, parse_length, RespDecode, RespEncode, RespError, CRLF_LEN};
use anyhow::Result;
use bytes::{Buf, BytesMut};
use std::ops::Deref;

const NULL_BULK_STRING: &str = "$-1\r\n";

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Default)]
pub struct BulkString(pub(crate) Option<Vec<u8>>);

impl BulkString {
  pub fn new(s: impl Into<Vec<u8>>) -> Self {
    BulkString(Some(s.into()))
  }
}

// - bulk string: "$<length>\r\n<data>\r\n"
impl RespEncode for BulkString {
  fn encode(self) -> Vec<u8> {
    match self {
      BulkString(Some(data)) => {
        let mut buf = Vec::with_capacity(data.len() + 16);
        buf.extend_from_slice(format!("${}\r\n", data.len()).as_bytes());
        buf.extend_from_slice(&data);
        buf.extend_from_slice(b"\r\n");
        buf
      }
      BulkString(None) => NULL_BULK_STRING.as_bytes().to_vec(),
    }
  }
}

impl RespDecode for BulkString {
  const PREFIX: &'static str = "$";
  fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
    let is_null = extract_fixed_data(buf, NULL_BULK_STRING, "NullBulkString").is_ok();
    if is_null {
      return Ok(BulkString::default());
    }

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
    BulkString(Some(s.to_vec()))
  }
}

impl From<String> for BulkString {
  fn from(s: String) -> Self {
    BulkString(Some(s.into_bytes()))
  }
}

impl From<&str> for BulkString {
  fn from(s: &str) -> Self {
    BulkString(Some(s.as_bytes().to_vec()))
  }
}

impl From<Option<&[u8]>> for BulkString {
  fn from(s: Option<&[u8]>) -> Self {
    match s {
      Some(s) => Self::new(Vec::from(s)),
      None => Self::default(),
    }
  }
}

impl AsRef<[u8]> for BulkString {
  fn as_ref(&self) -> &[u8] {
    match &self.0 {
      Some(s) => s,
      None => NULL_BULK_STRING.as_bytes(),
    }
  }
}

impl Deref for BulkString {
  type Target = Option<Vec<u8>>;

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

  #[test]
  fn test_null_bulk_string_encode() {
    let frame: RespFrame = BulkString(None).into();
    assert_eq!(frame.encode(), b"$-1\r\n");
  }

  #[test]
  fn test_null_bulk_string_decode() -> Result<()> {
    let mut buf = BytesMut::new();
    buf.extend_from_slice(b"$-1\r\n");

    let frame = BulkString::decode(&mut buf)?;
    assert_eq!(frame, BulkString(None));

    Ok(())
  }
}
