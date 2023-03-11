mod environment;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod typechecker;
mod types;

use error::Error;
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use typechecker::Typechecker;

/// Runs a ono program
pub fn run(program: &str) -> Result<(), Vec<Error>> {
    let tokens = Lexer::new().tokenize(program)?;
    let statements = Parser::new().parse(tokens)?;

    Typechecker::new().check(statements.clone())?;
    Interpreter::new().interpret(statements)?;
    Ok(())
}
