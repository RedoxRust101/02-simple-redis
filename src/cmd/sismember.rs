use super::{extract_args, validate_command, CommandError, CommandExecutor};
use crate::{Backend, RespArray, RespFrame};

///  SADD myset "one"
/// (integer) 1
///  SISMEMBER myset "one"
/// (integer) 1
/// SISMEMBER myset "two"
/// (integer) 0
#[derive(Debug)]
pub struct SIsMember {
  pub(crate) key: String,
  pub(crate) member: String,
}

impl CommandExecutor for SIsMember {
  fn execute(self, backend: &Backend) -> RespFrame {
    let is_member = backend.sismember(&self.key, &self.member);
    if is_member {
      RespFrame::Integer(1)
    } else {
      RespFrame::Integer(0)
    }
  }
}

impl TryFrom<RespArray> for SIsMember {
  type Error = CommandError;

  fn try_from(value: RespArray) -> Result<Self, Self::Error> {
    validate_command(&value, &["sismember"], 2)?;

    let mut args = extract_args(value, 1)?.into_iter();
    match (args.next(), args.next()) {
      (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(member))) => {
        Ok(SIsMember { key: key.into(), member: member.into() })
      }
      _ => Err(CommandError::InvalidArgument("Invalid key or member".to_string())),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{RespDecode, SAdd};
  use anyhow::Result;
  use bytes::BytesMut;

  #[test]
  fn test_sismember_from_resp_array() -> Result<()> {
    let mut buf = BytesMut::new();
    buf.extend_from_slice(b"*3\r\n$9\r\nsismember\r\n$5\r\nmykey\r\n$5\r\nmyval\r\n");

    let frame = RespArray::decode(&mut buf)?;
    let ret: SIsMember = frame.try_into()?;

    assert_eq!(ret.key, "mykey");
    assert_eq!(ret.member, "myval");

    Ok(())
  }

  #[test]
  fn test_sismember_execute() -> Result<()> {
    let backend = Backend::new();
    let cmd = SAdd { key: "mykey".to_string(), members: vec!["hello".to_string()] };
    let ret = cmd.execute(&backend);

    assert_eq!(ret, RespFrame::Integer(1));

    let cmd = SIsMember { key: "mykey".to_string(), member: "hello".to_string() };
    let ret = cmd.execute(&backend);

    assert_eq!(ret, RespFrame::Integer(1));

    let cmd = SIsMember { key: "mykey".to_string(), member: "world".to_string() };
    let ret = cmd.execute(&backend);

    assert_eq!(ret, RespFrame::Integer(0));

    Ok(())
  }
}
