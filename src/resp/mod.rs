mod decode;
mod encode;

use std::ops::Deref;

use bytes::BytesMut;
use enum_dispatch::enum_dispatch;
use thiserror::Error;

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
}

#[enum_dispatch(RespEncode)]
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum RespFrame {
  SimpleString(SimpleString),
  Null(RespNull),
  BulkString(BulkString),
  Array(RespArray),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct SimpleString(String);
#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct RespNull;
#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct BulkString(pub(crate) Vec<u8>);
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct RespArray(pub(crate) Vec<RespFrame>);

impl Deref for SimpleString {
  type Target = String;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Deref for BulkString {
  type Target = [u8];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Deref for RespArray {
  type Target = Vec<RespFrame>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<const N: usize> From<&[u8; N]> for BulkString {
  fn from(s: &[u8; N]) -> Self {
    BulkString(s.to_vec())
  }
}

impl<const N: usize> From<&[u8; N]> for RespFrame {
  fn from(s: &[u8; N]) -> Self {
    BulkString(s.to_vec()).into()
  }
}

impl SimpleString {
  pub fn new(s: impl Into<String>) -> Self {
    SimpleString(s.into())
  }
}

impl BulkString {
  pub fn new(s: impl Into<Vec<u8>>) -> Self {
    BulkString(s.into())
  }
}

impl RespArray {
  pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
    RespArray(s.into())
  }
}
