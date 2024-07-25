mod decode;
mod encode;

use std::ops::Deref;

use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait RespEncode {
    fn encode(self) -> Vec<u8>;
}

#[enum_dispatch(RespEncode)]
#[derive(Debug, PartialEq, PartialOrd)]
pub enum RespFrame {
    SimpleString(SimpleString),
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct SimpleString(String);

impl Deref for SimpleString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}
