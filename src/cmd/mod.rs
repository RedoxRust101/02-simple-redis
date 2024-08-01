mod map;

use crate::{Backend, RespArray, RespError, RespFrame, SimpleString};
use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use thiserror::Error;

lazy_static! {
  static ref RESP_OK: RespFrame = SimpleString::new("OK").into();
}

#[derive(Error, Debug)]
pub enum CommandError {
  #[error("Invalid command: {0}")]
  InvalidCommand(String),
  #[error("Invalid argument: {0}")]
  InvalidArgument(String),
  #[error("Resp error: {0}")]
  RespError(#[from] RespError),
  #[error("UTF-8 error: {0}")]
  Utf8Error(#[from] std::string::FromUtf8Error),
}

#[enum_dispatch]
pub trait CommandExecutor {
  fn execute(self, backend: &Backend) -> RespFrame;
}

#[enum_dispatch(CommandExecutor)]
#[derive(Debug)]
pub enum Command {
  Get(Get),
  Set(Set),

  Unrecognized(Unrecognized),
}

#[derive(Debug)]
pub struct Get {
  key: String,
}

#[derive(Debug)]
pub struct Set {
  key: String,
  value: RespFrame,
}

#[derive(Debug)]
pub struct Unrecognized;

impl TryFrom<RespFrame> for Command {
  type Error = CommandError;
  fn try_from(v: RespFrame) -> Result<Self, Self::Error> {
    match v {
      RespFrame::Array(array) => Command::try_from(array),
      _ => Err(CommandError::InvalidCommand("command must be an array".to_string())),
    }
  }
}

impl TryFrom<RespArray> for Command {
  type Error = CommandError;
  fn try_from(v: RespArray) -> Result<Self, Self::Error> {
    match v.first() {
      Some(RespFrame::BulkString(ref cmd)) => match cmd.as_ref() {
        b"get" => Ok(Get::try_from(v)?.into()),
        b"set" => Ok(Set::try_from(v)?.into()),
        _ => Err(CommandError::InvalidCommand(String::from_utf8_lossy(cmd.as_ref()).to_string())),
      },
      _ => Ok(Unrecognized.into()),
    }
  }
}

impl CommandExecutor for Unrecognized {
  fn execute(self, _: &Backend) -> RespFrame {
    RESP_OK.clone()
  }
}

fn validate_command(
  value: &RespArray,
  names: &[&'static str],
  n_arg: usize,
) -> Result<(), CommandError> {
  if value.len() != n_arg + names.len() {
    return Err(CommandError::InvalidArgument(format!(
      "expected {} arguments, got {}",
      n_arg + names.len(),
      value.len()
    )));
  }

  for (i, name) in names.iter().enumerate() {
    match value[i] {
      RespFrame::BulkString(ref cmd) => {
        if cmd.as_ref().to_ascii_lowercase() != name.as_bytes() {
          return Err(CommandError::InvalidCommand(format!(
            "expected command {}, got {}",
            name,
            String::from_utf8_lossy(cmd.as_ref())
          )));
        }
      }
      _ => {
        return Err(CommandError::InvalidCommand(
          "command must have a BulkString as the first argument2".to_string(),
        ))
      }
    }
  }
  Ok(())
}

fn extract_args(value: RespArray, start: usize) -> Result<Vec<RespFrame>, CommandError> {
  Ok(value.0.into_iter().skip(start).collect::<Vec<RespFrame>>())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{RespDecode, RespNull};
  use anyhow::Result;
  use bytes::BytesMut;

  #[test]
  fn test_command() -> Result<()> {
    let mut buf = BytesMut::new();
    buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");

    let frame = RespArray::decode(&mut buf)?;
    let cmd: Command = frame.try_into()?;
    let backend = Backend::new();
    let ret = cmd.execute(&backend);

    assert_eq!(ret, RespFrame::Null(RespNull));
    Ok(())
  }
}
