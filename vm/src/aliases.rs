use crate::err::RuntimeError;

pub static MAGIC_NUMBER: u64 = 448311488633;

pub type Result<T> = std::result::Result<T, RuntimeError>;
