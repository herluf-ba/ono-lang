use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

mod ast;
mod environment;
mod error;
mod functions;
mod interpreter;
mod lexer;
mod parser;
mod token;

use error::*;
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

struct Program {
    current_filename: Option<String>,
    lines: Vec<String>,
    lexer: Lexer,
    parser: Parser,
    interpreter: Interpreter,
}

impl Program {
    pub fn new() -> Self {
        Self {
            current_filename: None,
            lines: Vec::new(),
            lexer: Lexer::new(),
            parser: Parser::new(),
            interpreter: Interpreter::new(),
        }
    }

    pub fn feed(&mut self, src: String) -> Result<(), ()> {
        self.lines
            .extend(src.split('\n').map(String::from).collect::<Vec<String>>());

        let tokens = match self.lexer.tokenize(&src) {
            Ok(tokens) => tokens,
            Err(errors) => {
                self.report_errors(errors);
                return Err(());
            }
        };

        let statements = match self.parser.parse(tokens) {
            Ok(statements) => statements,
            Err(errors) => {
                self.report_errors(errors);
                return Err(());
            }
        };

        match self.interpreter.interpret(statements) {
            Ok(_) => Ok(()),
            Err(mut error) => {
                self.report_error(&mut error);
                return Err(());
            }
        }
    }

    fn feed_file(&mut self, filename: &str) -> Result<(), ()> {
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

        self.feed(src)
    }

    fn report_error(&self, error: &mut Error) {
        error.with_src_line(&self.lines[error.token.position.line]);

        if let Some(filename) = &self.current_filename {
            error.with_filename(&filename);
        }

        println!("{}", error);
    }

    fn report_errors(&self, errors: Vec<Error>) {
        for mut error in errors {
            self.report_error(&mut error);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        panic!("Usage: ono [script?]");
    }

    let mut program = Program::new();

    if args.len() == 2 {
        match program.feed_file(&args[1]) {
            Err(_) => {}
            Ok(_) => {}
        }
    } else {
        run_prompt(program);
    }
}

fn prompt_user() {
    print!("> ");
    std::io::stdout().flush().unwrap()
}

fn run_prompt(mut program: Program) {
    prompt_user();
    for line in std::io::stdin().lines() {
        let text = match line {
            Err(why) => panic!("couldn't read line: {}", why),
            Ok(text) => text,
        };

        match text.as_str() {
            "exit" => return (),
            _ => match program.feed(text) {
                Err(_) => {}
                Ok(_) => println!("{}", "todo"),
            },
        }

        prompt_user();
    }
}
