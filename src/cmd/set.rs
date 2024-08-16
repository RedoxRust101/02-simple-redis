use super::{
  extract_args, extract_bulk_string, validate_command, CommandError, CommandExecutor, RESP_OK,
};
use crate::{Backend, RespArray, RespFrame};

#[derive(Debug)]
pub struct Set {
  key: String,
  value: RespFrame,
}

impl CommandExecutor for Set {
  fn execute(self, backend: &Backend) -> RespFrame {
    backend.set(self.key, self.value);
    RESP_OK.clone()
  }
}

impl TryFrom<RespArray> for Set {
  type Error = CommandError;
  fn try_from(value: RespArray) -> Result<Self, Self::Error> {
    validate_command(&value, &["set"], 2)?;

    let mut args = extract_args(value, 1)?.into_iter();
    match (args.next(), args.next()) {
      (Some(RespFrame::BulkString(key)), Some(value)) => {
        Ok(Set { key: extract_bulk_string(key, "Invalid key")?, value })
      }
      _ => Err(CommandError::InvalidArgument("Invalid key or value".to_string())),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{Backend, Get, RespDecode};
  use anyhow::Result;
  use bytes::BytesMut;

  #[test]
  fn test_set_from_resp_array() -> Result<()> {
    let mut buf = BytesMut::new();
    buf.extend_from_slice(b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n");

    let frame = RespArray::decode(&mut buf)?;

    let ret: Set = frame.try_into()?;

    assert_eq!(ret.key, "hello");
    assert_eq!(ret.value, RespFrame::BulkString(b"world".into()));

    Ok(())
  }

  #[test]
  fn test_set_get_command() -> Result<()> {
    let backend = Backend::new();
    let cmd = Set { key: "hello".to_string(), value: RespFrame::BulkString(b"world".into()) };
    let ret = cmd.execute(&backend);

    assert_eq!(ret, RESP_OK.clone());

    let cmd = Get { key: "hello".to_string() };
    let ret = cmd.execute(&backend);

    assert_eq!(ret, RespFrame::BulkString(b"world".into()));

    Ok(())
  }
}
