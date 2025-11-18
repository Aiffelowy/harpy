use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: harpy <filename.hrpy>");
        process::exit(1);
    }

    let filename = &args[1];

    // Compile the source file
    let bytecode = match harpy_compiler::compile_file(filename) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Compilation failed: {:?}", e);
            process::exit(1);
        }
    };

    // Run the bytecode
    if let Err(e) = harpy_vm::run_bytecode(&bytecode) {
        eprintln!("Runtime error: {:?}", e);
        process::exit(1);
    }
}
