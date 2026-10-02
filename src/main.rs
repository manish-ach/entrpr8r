mod interpreter;

use interpreter::Interpreter;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut interpreter = Interpreter::new();

    match args.as_slice() {
        [_program] => interpreter.run_interactively(),
        [_program, path] => {
            if let Err(error) = interpreter.run_file(path) {
                eprintln!("Could not read {path}: {error}");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Usage: interpreter [file]");
            std::process::exit(2);
        }
    }
}
