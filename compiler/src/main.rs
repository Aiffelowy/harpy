use std::io::BufReader;

use harpy_compiler::{
    aliases::Result,
    analyzer::analyzer::Analyzer,
    lexer::Lexer,
    parser::{pretty_print::AstPrettyPrint, Parser},
    source::SourceFile,
};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file.hrpy>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let reader = BufReader::new(std::fs::File::open(filename)?);
    let source = SourceFile::new(reader)?;

    let lexer = Lexer::new(&source);
    let parser = Parser::new(lexer);

    let ast = match parser.build_ast() {
        Ok(a) => a,
        Err(errors) => {
            for error in errors {
                error.print_diagnostic(&source, filename);
            }
            return Ok(());
        }
    };

    let analyzer = Analyzer::default();
    let db = match analyzer.analyze(&ast) {
        Ok(db) => db,
        Err(e) => {
            for error in e {
                error.print_diagnostic(&source, filename);
            }
            return Ok(());
        }
    };

    println!("{}", AstPrettyPrint::new().print(&ast));

    print!("{:?}", db);

    Ok(())
}
