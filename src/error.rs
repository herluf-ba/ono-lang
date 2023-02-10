use colored::Colorize;
use std::fmt;

use crate::types::Token;

/// A static syntax error.
/// These are caught before running the program.
#[derive(Debug, PartialEq)]
pub enum SyntaxError {
    /// Unexpected symbol encountered
    S001,
    /// Unterminated string
    S002,
    /// Unterminated parenthesis
    S003,
    /// Expected expression
    S004,
}

/// A type error.
/// These are caught before running the program.
#[derive(Debug, PartialEq)]
pub enum TypeError {}

/// Runtime errors chrash the program.
#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    /// Division by zero
    R001,
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    Syntax(SyntaxError),
    Type(TypeError),
    Runtime(RuntimeError),
}

/// Standard ono error type
#[derive(Debug, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub token: Token,
    pub file: Option<String>,
    pub line_src: Option<String>,
}

/// panics with a formatted language error
pub fn language_error(message: &str) -> ! {
    panic!("{}", format!("{} {}", "[ONO LANGUAGE ERROR]".red().bold(), message))
}

impl Error {
    pub fn syntax_error(errno: SyntaxError, token: Token) -> Self {
        Self {
            kind: ErrorKind::Syntax(errno),
            token,
            file: None,
            line_src: None,
        }
    }

    pub fn type_error(errno: TypeError, token: Token) -> Self {
        Self {
            kind: ErrorKind::Type(errno),
            token,
            file: None,
            line_src: None,
        }
    }

    pub fn runtime_error(errno: RuntimeError, token: Token) -> Self {
        Self {
            kind: ErrorKind::Runtime(errno),
            token,
            file: None,
            line_src: None,
        }
    }

    pub fn with_src_line(&mut self, line_src: &str) {
        self.line_src = Some(line_src.to_string())
    }

    pub fn with_filename(&mut self, filename: &str) {
        self.file = Some(filename.to_string())
    }

    fn format_line_src(&self) -> String {
        match &self.line_src {
            None => String::new(),
            Some(src) => {
                let row_str = format!("{} | ", self.token.position.line + 1);
                let column_indicator = {
                    let spaces = std::iter::repeat(" ")
                        .take(row_str.len() + self.token.position.column - self.token.lexeme.len())
                        .collect::<String>();
                    let arrows = std::iter::repeat("^")
                        .take(self.token.lexeme.len())
                        .collect::<String>();

                    format!("{}{}", spaces, arrows.red().bold())
                };

                format!("{}{}\n{}", row_str.cyan(), src, column_indicator)
            }
        }
    }

    fn format_filename(&self) -> String {
        match &self.file {
            None => String::new(),
            Some(filename) => format!("-> {} {}", filename, self.token.position)
                .cyan()
                .to_string(),
        }
    }

    fn format_message(&self) -> String {
        let identifier = match &self.kind {
            ErrorKind::Syntax(_) => "error",
            ErrorKind::Type(_) => "type error",
            ErrorKind::Runtime(_) => "runtime error",
        }
        .bright_red();

        let message = match &self.kind {
            ErrorKind::Syntax(errno) => match errno {
                SyntaxError::S001 => {
                    format!("encountered unexpected symbol '{}'", self.token.lexeme)
                }
                SyntaxError::S002 => format!("unterminated string starting here"),
                SyntaxError::S003 => format!("unterminated parenthesis starting here"),
                SyntaxError::S004 => format!("expected expression after '{}'", self.token.lexeme),
            },
            ErrorKind::Type(errno) => match errno {
                _ => format!("{:#?}", errno),
            },
            ErrorKind::Runtime(errno) => match errno {
                _ => format!("{:#?}", errno),
            },
        };

        format!("{}: {}", identifier, message).bold().to_string()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\n{}\n{}\n{}",
            self.format_message(),
            self.format_filename(),
            self.format_line_src(),
        )
    }
}
