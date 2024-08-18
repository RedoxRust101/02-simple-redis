use super::{extract_args, validate_command, CommandError, CommandExecutor};
use crate::{Backend, RespArray, RespFrame};

/// redis> SADD myset "Hello"
/// (integer) 1
///  SADD myset "World"
/// (integer) 1
///  SMEMBERS myset
/// 1) "Hello"
/// 2) "World"
#[derive(Debug)]
pub struct SMembers {
  pub(crate) key: String,
}
impl CommandExecutor for SMembers {
  fn execute(self, backend: &Backend) -> RespFrame {
    let members = backend.smembers(&self.key);
    match members {
      Some(members) => {
        let ret = members
          .into_iter()
          .map(|member| RespFrame::BulkString(member.into()))
          .collect::<Vec<RespFrame>>();
        RespArray::new(ret).into()
      }
      None => RespArray::new([]).into(),
    }
  }
}

impl TryFrom<RespArray> for SMembers {
  type Error = crate::CommandError;
  fn try_from(value: RespArray) -> Result<Self, Self::Error> {
    validate_command(&value, &["smembers"], 1)?;

    let mut args = extract_args(value, 1)?.into_iter();
    match args.next() {
      Some(RespFrame::BulkString(key)) => Ok(SMembers { key: key.into() }),
      _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
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
  fn test_smembers_from_resp_array() -> Result<()> {
    let mut buf = BytesMut::new();
    buf.extend_from_slice(b"*2\r\n$8\r\nSMEMBERS\r\n$5\r\nhello\r\n");
    let frame = RespArray::decode(&mut buf)?;
    let cmd: SMembers = frame.try_into()?;
    assert_eq!(cmd.key, "hello");
    Ok(())
  }

  #[test]
  fn test_smembers_execute() -> Result<()> {
    let backend = Backend::new();
    let cmd =
      SAdd { key: "mykey".to_string(), members: vec!["hello".to_string(), "world".to_string()] };
    let ret = cmd.execute(&backend);

    assert_eq!(ret, RespFrame::Integer(2));

    let cmd = SMembers { key: "mykey".to_string() };
    let ret = cmd.execute(&backend);

    assert_eq!(
      ret,
      RespArray::new(vec![
        RespFrame::BulkString("hello".into()),
        RespFrame::BulkString("world".into())
      ])
      .into()
    );
    Ok(())
  }
}
