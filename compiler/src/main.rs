use std::io::BufReader;

use harpy_compiler::{aliases::Result, lexer::Lexer, source::SourceFile, tt};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file.hrpy>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let reader = BufReader::new(std::fs::File::open(filename)?);
    let source = SourceFile::new(reader)?;

    let mut lexer = Lexer::new(&source);

    while let Ok(token) = lexer.next_token() {
        println!("{:?}", token);
        if let tt!(eof) = lexer.peek()? {
            break;
        }
    }

    Ok(())
}
