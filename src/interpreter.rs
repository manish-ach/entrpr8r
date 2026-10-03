use std::io::{self, Write};

use crate::scanner::{self, Scanner};

fn read_line(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    input.trim().to_owned()
}

pub struct Interpreter {
    pub had_error: bool,
}

impl Interpreter {
    pub fn new() -> Self {
        Self { had_error: false }
    }

    pub fn error(&mut self, line: usize, msg: String) {
        eprintln!("[line {line}] Error: {msg}");
        self.had_error = true;
    }

    pub fn run_file(&mut self, path: &String) -> Result<(), std::io::Error> {
        let source = std::fs::read_to_string(path)?;
        self.run(&source);

        if self.had_error {
            self.error(0, "none for now".to_string());
            std::process::exit(65);
        }

        Ok(())
    }

    pub fn run_interactively(&mut self) {
        loop {
            let source = read_line(">");
            if source.is_empty() {
                continue;
            }
            if source == "exit".to_string() {
                println!("Exiting REPL mode");
                break;
            }
            self.run(&source);
            self.had_error = false;
        }
    }

    fn run(&mut self, source: &str) {
        let mut scanner = Scanner::new(source.to_owned());
        let tokens = scanner.scan_tokens(self);

        for token in tokens {
            println!("{token}");
        }
    }
}
