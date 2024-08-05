use super::{
  array::RespArray, bulk_string::BulkString, null::RespNull, simple_string::SimpleString,
  RespDecode, RespError,
};
use anyhow::Result;
use bytes::BytesMut;
use enum_dispatch::enum_dispatch;

#[enum_dispatch(RespEncode)]
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum RespFrame {
  SimpleString(SimpleString),
  Null(RespNull),
  BulkString(BulkString),
  Array(RespArray),
  Boolean(bool),
  Integer(i64),
  Double(f64),
}

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
      Some(b'*') => {
        let frame = RespArray::decode(buf)?;
        Ok(frame.into())
      }
      Some(b'_') => {
        let frame = RespNull::decode(buf)?;
        Ok(frame.into())
      }
      Some(b'#') => {
        let frame = bool::decode(buf)?;
        Ok(frame.into())
      }
      Some(b':') => {
        let frame = i64::decode(buf)?;
        Ok(frame.into())
      }
      Some(b',') => {
        let frame = f64::decode(buf)?;
        Ok(frame.into())
      }
      None => Err(RespError::NotComplete),
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
      Some(b'*') => RespArray::expect_length(buf),
      Some(b'_') => RespNull::expect_length(buf),
      Some(b'#') => bool::expect_length(buf),
      Some(b':') => i64::expect_length(buf),
      Some(b',') => f64::expect_length(buf),
      _ => Err(RespError::NotComplete),
    }
  }
}
/*
impl From<&str> for RespFrame {
  fn from(s: &str) -> Self {
    SimpleString::new(s).into()
  }
}

impl From<&[u8]> for RespFrame {
  fn from(s: &[u8]) -> Self {
    BulkString(s.to_vec()).into()
  }
}
 */
impl<const N: usize> From<&[u8; N]> for RespFrame {
  fn from(s: &[u8; N]) -> Self {
    BulkString(s.to_vec()).into()
  }
}

#[cfg(test)]
mod tests {
  // TODO: Add tests
}
