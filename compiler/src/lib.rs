use std::io::BufReader;

use aliases::Result;
use generator::generator::Generator;
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

pub fn compile_file(filename: &str) -> Result<Vec<u8>> {
    let reader = BufReader::new(std::fs::File::open(filename)?);
    let source = SourceFile::new(reader)?;

    let lexer = lexer::Lexer::new(&source)?;
    let parser = Parser::new(lexer);

    let ast = parser.build_ast().map_err(|errors| {
        for err in errors {
            err.show(&source);
        }
        std::io::Error::new(std::io::ErrorKind::Other, "Parse errors")
    })?;

    let result = Analyzer::analyze(&ast)
        .map_err(|errors| {
            for err in errors {
                err.show(&source);
            }
            std::io::Error::new(std::io::ErrorKind::Other, "Analysis errors")
        })?
        .into_runtime()
        .map_err(|errors| {
            for err in errors {
                err.show(&source);
            }
            std::io::Error::new(std::io::ErrorKind::Other, "Runtime errors")
        })?;

    Ok(Generator::compile(&ast, result))
}
