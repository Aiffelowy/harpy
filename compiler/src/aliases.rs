use crate::err::HarpyError;

pub type Result<T> = std::result::Result<T, Box<HarpyError>>;
