#[derive(Debug)]
pub enum ParseError {
    InvalidFileType,
    InvalidHeaderSize,
    UnknownTypeId,
    OutOfBounds,
}

#[derive(Debug)]
pub enum RuntimeError {
    ParserError(ParseError),
    OutOfBounds,
    StackOverflow,
    BadStack,
    InvalidOpcode,
    InvalidOperation,
    IO(std::io::Error),
}

impl From<std::io::Error> for RuntimeError {
    fn from(value: std::io::Error) -> Self {
        RuntimeError::IO(value)
    }
}

impl From<ParseError> for RuntimeError {
    fn from(value: ParseError) -> Self {
        RuntimeError::ParserError(value)
    }
}
