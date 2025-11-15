use std::io::Read;

use aliases::Result;
use parser::{
    byte_reader::ByteReader,
    header::{Header, HEADER_SIZE},
};

mod aliases;
mod err;
mod parser;
mod runtime;

pub fn run_bytecode(bytecode: &[u8]) -> Result<()> {
    let reader = ByteReader::new(bytecode, HEADER_SIZE);
    let header = Header::parse(reader)?;

    let mut runtime = header.into_runtime(bytecode)?;
    runtime.run()
}

pub fn run_file(filename: &str) -> Result<()> {
    let mut file = std::fs::File::open(filename)?;
    let mut bytes = vec![];
    file.read_to_end(&mut bytes)?;
    
    run_bytecode(&bytes)
}