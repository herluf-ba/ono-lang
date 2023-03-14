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
use types::Value;

/// Runs a ono program
pub fn run(program: &str) -> Result<Value, Vec<Error>> {
    let tokens = Lexer::new().tokenize(program)?;
    let statements = Parser::new().parse(tokens)?;

    Typechecker::new().check(&statements)?;
    Ok(Interpreter::new().interpret(&statements)?)
}
