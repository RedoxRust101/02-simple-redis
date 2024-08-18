use super::{extract_args, validate_command, CommandError, CommandExecutor};

use crate::{cmd::RESP_NULL, Backend, RespArray, RespFrame};
#[derive(Debug)]
pub struct Get {
  pub(crate) key: String,
}

impl CommandExecutor for Get {
  fn execute(self, backend: &Backend) -> RespFrame {
    match backend.get(&self.key) {
      Some(value) => value,
      None => RESP_NULL.clone(),
    }
  }
}

impl TryFrom<RespArray> for Get {
  type Error = CommandError;
  fn try_from(value: RespArray) -> Result<Self, Self::Error> {
    validate_command(&value, &["get"], 1)?;

    let mut args = extract_args(value, 1)?.into_iter();
    match args.next() {
      Some(RespFrame::BulkString(key)) => Ok(Get { key: key.into() }),
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
  fn test_get_from_resp_array() -> Result<()> {
    let mut buf = BytesMut::new();
    buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");

    let frame = RespArray::decode(&mut buf)?;

    let ret: Get = frame.try_into()?;

    assert_eq!(ret.key, "hello");

    Ok(())
  }
}
