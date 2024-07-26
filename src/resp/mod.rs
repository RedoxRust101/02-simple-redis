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
}

#[enum_dispatch(RespEncode)]
#[derive(Debug, PartialEq, PartialOrd)]
pub enum RespFrame {
  SimpleString(SimpleString),
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct SimpleString(String);

impl Deref for SimpleString {
  type Target = String;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl SimpleString {
  pub fn new(s: impl Into<String>) -> Self {
    SimpleString(s.into())
  }
}
