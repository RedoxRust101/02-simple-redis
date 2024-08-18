use super::{extract_args, validate_command, CommandError, CommandExecutor};
use crate::{cmd::RESP_NULL, Backend, RespArray, RespFrame};

#[derive(Debug)]
pub struct HMGet {
  pub(crate) key: String,
  pub(crate) fields: Vec<String>,
}

impl CommandExecutor for HMGet {
  fn execute(self, backend: &Backend) -> RespFrame {
    RespArray::new(
      self
        .fields
        .iter()
        .map(|field| backend.hget(&self.key, field).unwrap_or(RESP_NULL.clone()))
        .collect::<Vec<RespFrame>>(),
    )
    .into()
  }
}

impl TryFrom<RespArray> for HMGet {
  type Error = CommandError;
  fn try_from(value: RespArray) -> Result<Self, Self::Error> {
    validate_command(&value, &["hmget"], 2)?;

    let mut args = extract_args(value, 1)?.into_iter();
    match (args.next(), args.next()) {
      (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field))) => {
        let mut fields: Vec<String> = vec![field.into()];
        while let Some(RespFrame::BulkString(field)) = args.next() {
          fields.push(field.into());
        }
        Ok(HMGet { key: key.into(), fields })
      }
      _ => Err(CommandError::InvalidArgument("Invalid key or field".to_string())),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{cmd::RESP_OK, HSet, RespDecode};
  use anyhow::Result;
  use bytes::BytesMut;

  #[test]
  fn test_hmget_from_resp_array() -> Result<()> {
    let mut buf = BytesMut::new();
    buf.extend_from_slice(b"*4\r\n$5\r\nHMGET\r\n$5\r\nmykey\r\n$6\r\nfield1\r\n$6\r\nfield2\r\n");

    let frame = RespArray::decode(&mut buf)?;
    let ret: HMGet = frame.try_into()?;
    assert_eq!(ret.key, "mykey");
    Ok(())
  }

  #[test]
  fn test_hset_hmget_from_resp_array() -> Result<()> {
    let backend = Backend::new();
    let cmd = HSet {
      key: "map".to_string(),
      field: "hello".to_string(),
      value: RespFrame::BulkString(b"world".into()),
    };
    let ret = cmd.execute(&backend);

    assert_eq!(ret, RESP_OK.clone());

    let cmd =
      HMGet { key: "map".to_string(), fields: vec!["hello".to_string(), "field".to_string()] };
    let ret = cmd.execute(&backend);
    assert_eq!(
      ret,
      RespArray::new(vec![RespFrame::BulkString(b"world".into()), RESP_NULL.clone(),]).into()
    );
    Ok(())
  }
}
