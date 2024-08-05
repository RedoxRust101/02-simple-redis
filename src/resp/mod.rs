mod array;
mod bool;
mod bulk_string;
mod double;
mod frame;
mod integer;
mod null;
mod simple_string;

pub use self::{
  array::RespArray, bulk_string::BulkString, frame::RespFrame, null::RespNull,
  simple_string::SimpleString,
};
use bytes::{Buf, BytesMut};
use enum_dispatch::enum_dispatch;
use thiserror::Error;

const BUF_CAP: usize = 4096;
const CRLF: &[u8] = b"\r\n";
const CRLF_LEN: usize = CRLF.len();

#[enum_dispatch]
pub trait RespEncode {
  fn encode(self) -> Vec<u8>;
}

#[enum_dispatch]
pub trait RespDecode: Sized {
  const PREFIX: &'static str;
  fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;
  fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RespError {
  #[error("Invalid frame: {0}")]
  InvalidFrame(String),
  #[error("Invalid frame type: {0}")]
  InvalidFrameType(String),
  #[error("Invalid frame length: {0}")]
  InvalidFrameLength(isize),
  #[error("Frame is not complete")]
  NotComplete,
  #[error("Parse error: {0}")]
  ParseIntError(#[from] std::num::ParseIntError),
  #[error("Parse error: {0}")]
  ParseFloatError(#[from] std::num::ParseFloatError),
}

fn extract_fixed_data(
  buf: &mut BytesMut,
  expect: &str,
  expect_type: &str,
) -> Result<(), RespError> {
  if buf.len() < expect.len() {
    return Err(RespError::NotComplete);
  }

  if !buf.starts_with(expect.as_bytes()) {
    return Err(RespError::InvalidFrameType(format!("expect: {} got: {:?}", expect_type, buf)));
  }

  buf.advance(expect.len());
  Ok(())
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

#[cfg(test)]
mod tests {
  use super::*;
  use anyhow::Result;

  #[test]
  fn test_calc_array_length() -> Result<()> {
    let buf = b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n";
    let (end, len) = parse_length(buf, "*")?;
    let total_len = calc_total_length(buf, end, len, "*")?;
    assert_eq!(total_len, buf.len());

    let buf = b"*2\r\n$3\r\nget\r\r";
    let (end, len) = parse_length(buf, "*")?;
    let ret = calc_total_length(buf, end, len, "*");
    assert_eq!(ret.unwrap_err(), RespError::NotComplete);

    Ok(())
  }
}
