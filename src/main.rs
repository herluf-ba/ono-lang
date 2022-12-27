use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

mod ast;
mod error;
mod interpreter;
mod lexer;
mod parser;

use error::*;
use interpreter::Value;
use lexer::Lexer;
use parser::Parser;

struct Program {
    lines: Vec<String>,
    lexer: Lexer,
    parser: Parser,
    //interpreter: Interpreter,
}

impl Program {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            lexer: Lexer::new(),
            parser: Parser::new(),
            //interpreter: Interpreter {},
        }
    }

    pub fn feed(&mut self, src: String) -> Result<Value, ()> {
        self.lines
            .extend(src.split('\n').map(String::from).collect::<Vec<String>>());
        let tokens = match self.lexer.tokenize(&src) {
            Ok(tokens) => tokens,
            Err(errors) => {
                self.report_errors(errors);
                return Err(());
            }
        };

        let ast = match self.parser.parse(tokens) {
            Ok(ast) => ast,
            Err(mut error) => {
                self.report_error(&mut error);
                return Err(());
            }
        };

        println!("{:#?}", ast);

        // TODO: Interpret

        Ok(Value::Null)
    }

    fn report_error(&self, error: &mut Error) {
        // TODO: Test error output here
        if let Some(row) = error.row {
            error.add_src(&self.lines[row]);
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

    let program = Program::new();

    if args.len() == 2 {
        run_file(program, &args[1]);
    } else {
        run_prompt(program);
    }
}

fn run_file(mut program: Program, filename: &str) {
    let path = Path::new(&filename);

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };

    let mut src = String::new();
    match file.read_to_string(&mut src) {
        Err(why) => panic!("couldn't read {}: {}", path.display(), why),
        Ok(_) => {}
    };

    match program.feed(src) {
        Ok(_) => {}
        Err(_) => {} // TODO: Exit with some code
    };
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
                Ok(value) => println!("{}", value),
            },
        }

        prompt_user();
    }
}
