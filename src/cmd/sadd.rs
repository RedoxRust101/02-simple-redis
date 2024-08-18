use super::CommandExecutor;
use super::{extract_args, validate_command, CommandError};
use crate::{Backend, RespArray, RespFrame};

const INT_1: i64 = 1;
const INT_0: i64 = 0;

/// SADD key member [member ...]
/// SADD myset "Hello"
/// (integer) 1
///  SADD myset "World"
/// (integer) 1
///  SADD myset "World"
/// (integer) 0
///  SMEMBERS myset
/// 1) "Hello"
/// 2) "World"
#[derive(Debug)]
pub struct SAdd {
  pub(crate) key: String,
  pub(crate) members: Vec<String>,
}

impl CommandExecutor for SAdd {
  fn execute(self, backend: &Backend) -> RespFrame {
    let ret = self
      .members
      .iter()
      .map(|member| if backend.sadd(&self.key, member) { INT_1 } else { INT_0 })
      .sum::<i64>();

    RespFrame::Integer(ret)
  }
}

impl TryFrom<RespArray> for SAdd {
  type Error = CommandError;

  fn try_from(value: RespArray) -> Result<Self, Self::Error> {
    validate_command(&value, &["sadd"], 2)?;
    let mut args = extract_args(value, 1)?.into_iter();
    match (args.next(), args.next()) {
      (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(member))) => {
        let mut members = vec![member.into()];
        while let Some(RespFrame::BulkString(member)) = args.next() {
          members.push(member.into());
        }
        Ok(SAdd { key: key.into(), members })
      }
      _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::RespDecode;
  use anyhow::Result;
  use bytes::BytesMut;

  #[test]
  fn test_sadd_from_resp_array() -> Result<()> {
    let mut buf = BytesMut::new();
    buf.extend_from_slice(b"*3\r\n$4\r\nsadd\r\n$5\r\nmykey\r\n$7\r\nmyvalue\r\n");
    let frame = RespArray::decode(&mut buf)?;
    let cmd: SAdd = frame.try_into()?;

    assert_eq!(cmd.key, "mykey");
    assert_eq!(cmd.members, vec!["myvalue".to_string()]);

    Ok(())
  }

  #[test]
  fn test_sadd_execute() -> Result<()> {
    let backend = Backend::new();
    let cmd =
      SAdd { key: "mykey".to_string(), members: vec!["hello".to_string(), "world".to_string()] };
    let ret = cmd.execute(&backend);

    assert_eq!(ret, RespFrame::Integer(2));

    let cmd = SAdd { key: "mykey".to_string(), members: vec!["world".to_string()] };
    let ret = cmd.execute(&backend);

    assert_eq!(ret, RespFrame::Integer(0));

    Ok(())
  }
}
