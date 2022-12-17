use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

mod ast;
mod error;
mod lexer;

use error::*;

fn main() {
    //let top = Expr::Binary {
    //operator: Token {
    //kind: TokenKind::PLUS,
    //lexeme: "+".to_string(),
    //row: 0,
    //column: 1,
    //},
    //left: Box::new(Expr::Literal {
    //value: Token {
    //kind: TokenKind::NUMBER(1.0),
    //lexeme: "1".to_string(),
    //row: 0,
    //column: 0,
    //},
    //}),
    //right: Box::new(Expr::Literal {
    //value: Token {
    //kind: TokenKind::NUMBER(2.0),
    //lexeme: "2".to_string(),
    //row: 0,
    //column: 2,
    //},
    //}),
    //};
    //println!("{:#?}", top);
    //return;

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

    Ok(())
}
