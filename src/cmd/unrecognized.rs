use super::{CommandExecutor, RESP_OK};
use crate::{Backend, RespFrame};

#[derive(Debug)]
pub struct Unrecognized;

impl CommandExecutor for Unrecognized {
  fn execute(self, _: &Backend) -> RespFrame {
    RESP_OK.clone()
  }
}
