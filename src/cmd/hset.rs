use super::{extract_args, validate_command, CommandError, CommandExecutor, RESP_OK};
use crate::{Backend, RespArray, RespFrame};

#[derive(Debug)]
pub struct HSet {
  pub(crate) key: String,
  pub(crate) field: String,
  pub(crate) value: RespFrame,
}

impl CommandExecutor for HSet {
  fn execute(self, backend: &Backend) -> RespFrame {
    backend.hset(self.key, self.field, self.value);
    RESP_OK.clone()
  }
}

impl TryFrom<RespArray> for HSet {
  type Error = CommandError;
  fn try_from(value: RespArray) -> Result<Self, Self::Error> {
    validate_command(&value, &["hset"], 3)?;

    let mut args = extract_args(value, 1)?.into_iter();
    match (args.next(), args.next(), args.next()) {
      (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field)), Some(value)) => {
        Ok(HSet { key: key.into(), field: field.into(), value })
      }
      _ => Err(CommandError::InvalidArgument("Invalid key, field or value".to_string())),
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::{HSet, RespDecode};
  use anyhow::Result;
  use bytes::BytesMut;

  #[test]
  fn test_hset_from_resp_array() -> Result<()> {
    let mut buf = BytesMut::new();
    buf.extend_from_slice(b"*4\r\n$4\r\nhset\r\n$3\r\nmap\r\n$5\r\nhello\r\n$5\r\nworld\r\n");

    let frame = RespArray::decode(&mut buf)?;

    let ret: HSet = frame.try_into()?;
    assert_eq!(ret.key, "map");
    assert_eq!(ret.field, "hello");
    assert_eq!(ret.value, RespFrame::BulkString(b"world".into()));

    Ok(())
  }
}
