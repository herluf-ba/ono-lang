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
use interpreter::{Interpreter, Value};
use parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        panic!("Usage: ono [script?]");
    }

    if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}

fn run_file(filename: &String) {
    let path = Path::new(filename);

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };

    let mut src = String::new();
    match file.read_to_string(&mut src) {
        Err(why) => panic!("couldn't read {}: {}", path.display(), why),
        Ok(_) => {}
    };

    let mut interpreter = Interpreter {};
    match run(&mut interpreter, &src) {
        Ok(_) => {}
        Err(_) => {} // TODO: Exit with some code
    };
}

fn prompt_user() {
    print!("> ");
    std::io::stdout().flush().unwrap()
}

fn run_prompt() {
    let mut interpreter = Interpreter {};
    prompt_user();
    for line in std::io::stdin().lines() {
        let text = match line {
            Err(why) => panic!("couldn't read line: {}", why),
            Ok(text) => text,
        };

        match text.as_str() {
            "exit" => return (),
            _ => match run(&mut interpreter, &text) {
                Err(_) => {}
                Ok(val) => println!("{}", val),
            },
        }

        prompt_user();
    }
}

fn run(interpreter: &mut Interpreter, src: &str) -> Result<Value, ()> {
    let error_producer = ErrorProducer::new(src);

    let mut lexer = lexer::Lexer::new(src);
    let tokens = lexer.tokenize();
    if lexer.has_errors() {
        lexer.report_errors();
        return Err(());
    }

    let mut parser = Parser::from(tokens, &error_producer);
    let ast = match parser.parse() {
        Some(ast) => ast,
        None => {
            parser.report_errors();
            return Err(());
        }
    };

    match interpreter.interpret(&ast) {
        Value::Error { message, token } => {
            println!(
                "{}",
                error_producer.runtime_error_from_token(&token, &message)
            );
            Err(())
        }
        val => Ok(val),
    }
}
