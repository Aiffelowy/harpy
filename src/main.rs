use std::io::BufReader;

use aliases::Result;
use parser::parser::Parser;
use semantic_analyzer::analyzer::Analyzer;
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

    let ast = match parser.build_ast() {
        Ok(ast) => ast,
        Err(errors) => {
            for err in errors {
                err.show(&source);
            }
            return Ok(());
        }
    };

    match Analyzer::analyze(&ast) {
        Ok(()) => (),
        Err(errors) => {
            for err in errors {
                err.show(&source);
            }
            return Ok(());
        }
    }

    Ok(())
}
