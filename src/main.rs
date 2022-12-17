use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

mod ast;
mod error;
mod lexer;
mod parser;

use error::*;
use parser::AstBuilder;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        panic!("Usage: ono [script?]");
    }

    if args.len() == 2 {
        match run_file(&args[1]) {
            Err(_) => panic!(""),
            Ok(_) => return,
        }
    }

    run_prompt();
}

fn run_file(filename: &String) -> Result<()> {
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

    run(&src)
}

fn prompt_user() {
    print!("> ");
    std::io::stdout().flush().unwrap()
}

fn run_prompt() {
    prompt_user();
    for line in std::io::stdin().lines() {
        let text = match line {
            Err(why) => panic!("couldn't read line: {}", why),
            Ok(text) => text,
        };

        match text.as_str() {
            "exit" => return (),
            _ => match run(&text) {
                Err(_) => {}
                Ok(_) => {}
            },
        }

        prompt_user();
    }
}

fn run(src: &str) -> Result<()> {
    let mut lexer = lexer::Lexer::new(src);
    lexer.tokenize();

    if !lexer.is_ok() {
        lexer.report_errors();
        return Err(());
    }

    let mut ast_builder = AstBuilder::from(&lexer);
    let ast = match ast_builder.build() {
        Some(ast) => ast,
        None => {
            ast_builder.report_errors();
            return Err(());
        }
    };

    println!("{}", ast);

    Ok(())
}
