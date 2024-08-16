use super::{extract_args, validate_command, CommandError, CommandExecutor};
use crate::{Backend, BulkString, RespArray, RespFrame};
use std::vec;

#[derive(Debug)]
pub struct HGetAll {
  key: String,
  sort: bool,
}

impl CommandExecutor for HGetAll {
  fn execute(self, backend: &Backend) -> RespFrame {
    let hmap = backend.hgetall(&self.key);
    match hmap {
      Some(hmap) => {
        let mut data = Vec::with_capacity(hmap.len());
        for v in hmap.iter() {
          let key = v.key().to_owned();
          data.push((key, v.value().clone()));
        }
        if self.sort {
          data.sort_by(|a, b| a.0.cmp(&b.0));
        }
        let ret = data
          .into_iter()
          .flat_map(|(k, v)| vec![BulkString::from(k).into(), v])
          .collect::<Vec<RespFrame>>();

        RespArray::new(ret).into()
      }
      None => RespArray::new([]).into(),
    }
  }
}

impl TryFrom<RespArray> for HGetAll {
  type Error = CommandError;
  fn try_from(value: RespArray) -> Result<Self, Self::Error> {
    validate_command(&value, &["hgetall"], 1)?;

    let mut args = extract_args(value, 1)?.into_iter();
    match args.next() {
      Some(RespFrame::BulkString(key)) => Ok(HGetAll {
        key: match key.0 {
          Some(key) => String::from_utf8(key)?,
          None => return Err(CommandError::InvalidArgument("Invalid key".to_string())),
        },
        sort: false,
      }),
      _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::{cmd::RESP_OK, HGet, HSet, RespDecode};
  use anyhow::Result;
  use bytes::BytesMut;

  #[test]
  fn test_hgetall_from_resp_array() -> Result<()> {
    let mut buf = BytesMut::new();
    buf.extend_from_slice(b"*2\r\n$7\r\nhgetall\r\n$3\r\nmap\r\n");

    let frame = RespArray::decode(&mut buf)?;

    let ret: HGetAll = frame.try_into()?;
    assert_eq!(ret.key, "map");
    assert!(!ret.sort);

    Ok(())
  }

  #[test]
  fn test_hset_hget_hgetall_commands() -> Result<()> {
    let backend = Backend::new();
    let cmd = HSet {
      key: "map".to_string(),
      field: "hello".to_string(),
      value: RespFrame::BulkString(b"world".into()),
    };
    let ret = cmd.execute(&backend);

    assert_eq!(ret, RESP_OK.clone());

    let cmd = HSet {
      key: "map".to_string(),
      field: "hello1".to_string(),
      value: RespFrame::BulkString(b"world1".into()),
    };
    cmd.execute(&backend);

    let cmd = HGet { key: "map".to_string(), field: "hello".to_string() };
    let ret = cmd.execute(&backend);

    assert_eq!(ret, RespFrame::BulkString(b"world".into()));

    let cmd = HGetAll { key: "map".to_string(), sort: true };
    let ret = cmd.execute(&backend);

    assert_eq!(
      ret,
      RespArray::new([
        BulkString::from("hello").into(),
        BulkString::from("world").into(),
        BulkString::from("hello1").into(),
        BulkString::from("world1").into()
      ])
      .into()
    );

    Ok(())
  }
}
