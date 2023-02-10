use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

mod error;
mod interpreter;
mod lexer;
mod parser;
mod types;

use error::Error;
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

/// Represents an entire program
pub struct Program {
    current_filename: Option<String>,
    lines: Vec<String>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            current_filename: None,
            lines: Vec::new(),
        }
    }

    /// Run an ono program from a source file.
    /// Any errors are reported to stdout.
    pub fn run(&mut self, filename: &str) -> Result<(), Vec<Error>> {
        self.current_filename = Some(filename.to_string());
        let path = Path::new(&filename);
        let mut file = match File::open(&path) {
            Err(_) => return Err(Vec::new()), // TODO: create error for this
            Ok(file) => file,
        };

        let mut src = String::new();
        match file.read_to_string(&mut src) {
            Err(_) => return Err(Vec::new()), // TODO: create error for this
            Ok(_) => {}
        };

        self.lines
            .extend(src.split('\n').map(String::from).collect::<Vec<String>>());

        let mut lexer = Lexer::new();
        let tokens = match lexer.tokenize(&src) {
            Ok(tokens) => tokens,
            Err(mut errors) => {
                self.format_errors(&mut errors);
                return Err(errors);
            }
        };

        println!("{:#?}", tokens);

        let expr = match Parser::new().parse(tokens) {
            Ok(expr) => expr,
            Err(err) => {
                let mut errors = vec![err];
                self.format_errors(&mut errors);
                return Err(errors);
            }
        };

        println!("{:#?}", expr);

        let mut interpreter = Interpreter::new();
        let result = match interpreter.interpret(expr) {
            Ok(value) => value,
            Err(err) => {
                let mut errors = vec![err];
                self.format_errors(&mut errors);
                return Err(errors);
            }
        };

        println!("{:#?}", result);

        Ok(())
    }

    /// Adds line src and filename to errors
    fn format_errors(&self, errors: &mut Vec<Error>) {
        for error in errors.iter_mut() {
            error.with_src_line(&self.lines[error.token.position.line]);
            if let Some(filename) = &self.current_filename {
                error.with_filename(&filename);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("Usage: ono [script?]");
    }

    match Program::new().run(&args[1]) {
        Ok(_) => {}
        Err(errors) => {
            for error in errors.iter() {
                println!("{}", error);
            }
            println!(
                "Program exited with {} error{}",
                errors.len(),
                if errors.len() > 1 { "s" } else { "" }
            )
        }
    }
}
