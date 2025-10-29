use std::io::{BufReader, Read};

use aliases::Result;
use parser::parser::Parser;

pub mod aliases;
pub mod err;
pub mod lexer;
pub mod parser;

fn main() -> Result<()> {
    let mut reader = BufReader::new(std::fs::File::open("code.hrpy")?);
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;
    let lexer = lexer::Lexer::new(&buffer)?;
    let parser = Parser::new(lexer);

    println!("{:?}", parser.build_ast()?);

    Ok(())
}
