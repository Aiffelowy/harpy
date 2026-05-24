use harpy_compiler::{
    aliases::Result, analyzer::analyzer::Analyzer, err::ErrorPrinter, lexer::Lexer, parser::Parser,
    source::source_map::SourceMap,
};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file.hrpy>", args[0]);
        std::process::exit(1);
    }

    let mut source_map = SourceMap::default();

    let file_id = source_map.load_source(args[1].clone())?;
    let lexer = Lexer::new(source_map.get_source(file_id));
    let parser = Parser::new(lexer);

    let ast = match parser.build_ast() {
        Ok(a) => a,
        Err(errors) => {
            ErrorPrinter::print_all(&source_map, None, &errors);
            return Ok(());
        }
    };

    let analyzer = Analyzer::default();
    let db = match analyzer.analyze(&ast) {
        Ok(db) => db,
        Err((db, errors)) => {
            ErrorPrinter::print_all(&source_map, Some(&db), &errors);
            return Ok(());
        }
    };
    println!("{db}");
    Ok(())
}
