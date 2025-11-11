use std::io::BufReader;

use aliases::Result;
use err::HarpyError;
use parser::parser::Parser;
use semantic_analyzer::analyzer::Analyzer;
use source::SourceFile;

pub mod aliases;
pub mod color;
pub mod err;
pub mod extensions;
pub mod generator;
pub mod lexer;
pub mod parser;
pub mod semantic_analyzer;
pub mod source;

fn print_errors(errors: Vec<HarpyError>, source: &SourceFile) {
    for err in errors {
        err.show(source);
    }
}

fn main() -> Result<()> {
    let reader = BufReader::new(std::fs::File::open("code.hrpy")?);
    let source = SourceFile::new(reader)?;

    let lexer = lexer::Lexer::new(&source)?;
    let parser = Parser::new(lexer);

    let ast = match parser.build_ast() {
        Ok(ast) => ast,
        Err(errors) => {
            print_errors(errors, &source);
            return Ok(());
        }
    };

    match Analyzer::analyze(&ast) {
        Ok(result) => {
            let result = result.into_runtime()?;
            println!("TYPE TABLE:\n{:?}\n\n", result.type_table);
            println!("CONST POOL:\n{:?}\n\n", result.constants);
            println!("FUNCTION TABLE:\n{:?}", result.function_table);
        }
        Err(errors) => {
            print_errors(errors, &source);
            return Ok(());
        }
    }

    Ok(())
}
