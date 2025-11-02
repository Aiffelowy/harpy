use std::io::BufReader;

use aliases::Result;
use parser::parser::Parser;
use source::SourceFile;

pub mod aliases;
pub mod color;
pub mod err;
pub mod lexer;
pub mod parser;
pub mod semantic_analyzer;
pub mod source;

fn main() -> Result<()> {
    let reader = BufReader::new(std::fs::File::open("code.hrpy")?);
    let source = SourceFile::new(reader)?;

    let lexer = lexer::Lexer::new(&source)?;
    let parser = Parser::new(lexer);

    match parser.build_ast() {
        Ok(ast) => println!("AST: {:?}", ast),
        Err(errors) => {
            for err in errors {
                err.show(&source);
            }
        }
    }

    Ok(())
}
