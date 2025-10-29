use std::io::{BufReader, Read};

use aliases::Result;
use parser::func_decl::FuncDelc;

pub mod aliases;
pub mod err;
pub mod lexer;
pub mod parser;

fn main() -> Result<()> {
    let mut reader = BufReader::new(std::fs::File::open("code.hrpy")?);
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;
    let mut lexer = lexer::Lexer::new(&buffer)?;

    println!("{:?}", lexer.parse::<FuncDelc>()?);

    Ok(())
}
