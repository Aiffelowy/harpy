use std::io::{BufReader, Read};

use aliases::Result;
use parser::parser::Parser;

pub mod aliases;
pub mod color;
pub mod err;
pub mod lexer;
pub mod parser;

fn main() -> Result<()> {
    let mut reader = BufReader::new(std::fs::File::open("code.hrpy")?);
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;

    let lexer = lexer::Lexer::new(&buffer)?;
    let parser = Parser::new(lexer);

    match parser.build_ast() {
        Ok(ast) => println!("AST: {:?}", ast),
        Err(errors) => {
            for err in errors {
                err.show(&buffer);
            }
        }
    }

    Ok(())
}
