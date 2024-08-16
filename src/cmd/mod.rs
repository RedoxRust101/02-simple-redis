mod hmap;
mod map;

use crate::{Backend, BulkString, RespArray, RespError, RespFrame, SimpleString};
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
  HGet(HGet),
  HSet(HSet),
  HGetAll(HGetAll),

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
pub struct HGet {
  key: String,
  field: String,
}

#[derive(Debug)]
pub struct HSet {
  key: String,
  field: String,
  value: RespFrame,
}

#[derive(Debug)]
pub struct HGetAll {
  key: String,
  sort: bool,
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
    match &v.0 {
      Some(frames) => match frames.first() {
        Some(RespFrame::BulkString(ref cmd)) => match cmd.as_ref() {
          b"get" => Ok(Get::try_from(v)?.into()),
          b"set" => Ok(Set::try_from(v)?.into()),
          b"hget" => Ok(HGet::try_from(v)?.into()),
          b"hset" => Ok(HSet::try_from(v)?.into()),
          b"hgetall" => Ok(HGetAll::try_from(v)?.into()),
          _ => Ok(Unrecognized.into()),
        },
        _ => Err(CommandError::InvalidCommand("command must be an RespFrame".to_string())),
      },
      _ => Err(CommandError::InvalidCommand("command must be an RespArray".to_string())),
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
  let frames = extract_resp_array(value.clone(), "Invalid command.")?;
  if frames.len() != n_arg + names.len() {
    return Err(CommandError::InvalidArgument(format!(
      "expected {} arguments, got {}",
      n_arg + names.len(),
      frames.len()
    )));
  }

  for (i, name) in names.iter().enumerate() {
    match frames[i] {
      RespFrame::BulkString(ref s) => {
        let cmd = extract_bulk_string(s.clone(), "Invalid command")?;
        if cmd.as_bytes().to_ascii_lowercase() != name.as_bytes() {
          return Err(CommandError::InvalidCommand(format!(
            "expected command {}, got {}",
            name, cmd
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
  let frames = extract_resp_array(value, "Invalid args.")?;
  Ok(frames.into_iter().skip(start).collect::<Vec<RespFrame>>())
}

fn extract_bulk_string(s: BulkString, err_msg: &str) -> Result<String, CommandError> {
  match s.0 {
    Some(key) => Ok(String::from_utf8(key)?),
    None => Err(CommandError::InvalidArgument(err_msg.to_string())),
  }
}

fn extract_resp_array(value: RespArray, err_msg: &str) -> Result<Vec<RespFrame>, CommandError> {
  match value.0 {
    Some(arr) => Ok(arr),
    _ => Err(CommandError::InvalidArgument(err_msg.to_string())),
  }
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
