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

fn main() -> Result<()> {
    let filename = std::env::args().nth(1);
    if filename.is_none() {
        eprintln!("Usage: vm <file.hrpc>");
        std::process::exit(1);
    }

    let mut file = std::fs::File::open(filename.unwrap())?;
    let mut bytes = vec![];
    file.read_to_end(&mut bytes)?;

    let reader = ByteReader::new(&bytes, HEADER_SIZE);
    let header = Header::parse(reader)?;

    let mut runtime = header.into_runtime(&bytes)?;

    runtime.run()
}
