mod echo;
mod get;
mod hget;
mod hgetall;
mod hmget;
mod hset;
mod sadd;
mod set;
mod sismember;
mod smembers;
mod unrecognized;

pub use self::{
  echo::Echo, get::Get, hget::HGet, hgetall::HGetAll, hmget::HMGet, hset::HSet, sadd::SAdd,
  set::Set, sismember::SIsMember, smembers::SMembers, unrecognized::Unrecognized,
};
use crate::{Backend, RespArray, RespError, RespFrame, RespNull, SimpleString};
use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use thiserror::Error;

lazy_static! {
  static ref RESP_OK: RespFrame = SimpleString::new("OK").into();
  static ref RESP_NULL: RespFrame = RespFrame::Null(RespNull);
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
  Echo(Echo),
  HMGet(HMGet),
  SADD(SAdd),
  SMEMBERS(SMembers),
  SISMEMBER(SIsMember),

  Unrecognized(Unrecognized),
}

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
          b"echo" => Ok(Echo::try_from(v)?.into()),
          b"hmget" => Ok(HMGet::try_from(v)?.into()),
          b"sadd" => Ok(SAdd::try_from(v)?.into()),
          b"smembers" => Ok(SMembers::try_from(v)?.into()),
          b"sismember" => Ok(SIsMember::try_from(v)?.into()),
          _ => Ok(Unrecognized.into()),
        },
        _ => Err(CommandError::InvalidCommand("command must be an RespFrame".to_string())),
      },
      _ => Err(CommandError::InvalidCommand("command must be an RespArray".to_string())),
    }
  }
}
fn validate_command(
  value: &RespArray,
  names: &[&'static str],
  n_arg: usize,
) -> Result<(), CommandError> {
  let frames = extract_resp_array(value.clone(), "Invalid command.")?;
  if frames.len() < n_arg + names.len() {
    return Err(CommandError::InvalidArgument(format!(
      "expected at least {} arguments, got {}",
      n_arg + names.len(),
      frames.len()
    )));
  }

  for (i, name) in names.iter().enumerate() {
    match frames[i] {
      RespFrame::BulkString(ref s) => {
        let cmd: String = s.clone().into();
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

fn extract_resp_array(value: RespArray, err_msg: &str) -> Result<Vec<RespFrame>, CommandError> {
  match value.0 {
    Some(arr) => Ok(arr),
    _ => Err(CommandError::InvalidArgument(err_msg.to_string())),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::RespDecode;
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

    assert_eq!(ret, RESP_NULL.clone());
    Ok(())
  }
}
