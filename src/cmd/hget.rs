use super::{extract_args, validate_command, CommandError, CommandExecutor};
use crate::{cmd::RESP_NULL, Backend, RespArray, RespFrame};

#[derive(Debug)]
pub struct HGet {
  pub(crate) key: String,
  pub(crate) field: String,
}

impl CommandExecutor for HGet {
  fn execute(self, backend: &Backend) -> RespFrame {
    match backend.hget(&self.key, &self.field) {
      Some(value) => value,
      None => RESP_NULL.clone(),
    }
  }
}

impl TryFrom<RespArray> for HGet {
  type Error = CommandError;
  fn try_from(value: RespArray) -> Result<Self, Self::Error> {
    validate_command(&value, &["hget"], 2)?;

    let mut args = extract_args(value, 1)?.into_iter();
    match (args.next(), args.next()) {
      (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field))) => {
        Ok(HGet { key: key.into(), field: field.into() })
      }
      _ => Err(CommandError::InvalidArgument("Invalid key or field".to_string())),
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::RespDecode;
  use anyhow::Result;
  use bytes::BytesMut;

  #[test]

  fn test_hget_from_resp_array() -> Result<()> {
    let mut buf = BytesMut::new();
    buf.extend_from_slice(b"*3\r\n$4\r\nhget\r\n$3\r\nmap\r\n$5\r\nhello\r\n");

    let frame = RespArray::decode(&mut buf)?;

    let ret: HGet = frame.try_into()?;
    assert_eq!(ret.key, "map");
    assert_eq!(ret.field, "hello");

    Ok(())
  }
}
