use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

mod error;
mod lexer;
mod types;

use error::Error;
use lexer::Lexer;

/// Represents an entire program (may span multiple files).
pub struct Program {
    current_filename: Option<String>,
    lines: Vec<String>,
    lexer: Lexer,
}

impl Program {
    pub fn new() -> Self {
        Self {
            current_filename: None,
            lines: Vec::new(),
            lexer: Lexer::new(),
        }
    }

    /// Feed a file to the program. It is resolved and type checked immidiately.
    /// Referenced files are automatically gathered and consumed too.
    /// Any errors are reported to stdout.
    /// The entire program is valid and ready to `run` when this has finished.
    pub fn feed_file(&mut self, filename: &str) -> Result<(), ()> {
        self.current_filename = Some(filename.to_string());
        let path = Path::new(&filename);
        let mut file = match File::open(&path) {
            Err(_) => return Err(()), // TODO: exit code
            Ok(file) => file,
        };

        let mut src = String::new();
        match file.read_to_string(&mut src) {
            Err(_) => return Err(()), // TODO: Exit code
            Ok(_) => {}
        };

        self.lines
            .extend(src.split('\n').map(String::from).collect::<Vec<String>>());

        let tokens = match self.lexer.tokenize(&src) {
            Ok(tokens) => tokens,
            Err(errors) => {
                self.report_errors(errors);
                return Err(());
            }
        };

        println!("{:#?}", tokens);

        Ok(())
    }

    pub fn run(&self) -> Result<(), ()> {
        unimplemented!();
    }

    fn report_error(&self, error: &mut Error) {
        error.with_src_line(&self.lines[error.token.position.line]);

        if let Some(filename) = &self.current_filename {
            error.with_filename(&filename);
        }

        println!("{}", error);
    }

    fn report_errors(&self, mut errors: Vec<Error>) {
        for mut error in &mut errors {
            self.report_error(&mut error);
        }

        println!("Program exited with {} errors", errors.len())
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        panic!("Usage: ono [script?]");
    }

    let mut program = Program::new();
    if program.feed_file(&args[1]).is_ok() {
        match program.run() {
            Ok(_) => {}
            Err(_) => {}
        }
    }
}
