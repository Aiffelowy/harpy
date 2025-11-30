use crate::err::RuntimeError;

pub static MAGIC_NUMBER: [u8; 5] = [0x68, 0x61, 0x72, 0x70, 0x79];
pub static VERSION: u16 = 0x1;

pub type Result<T> = std::result::Result<T, RuntimeError>;
