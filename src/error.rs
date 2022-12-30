use colored::Colorize;
use std::fmt;

use crate::{interpreter::Value, lexer::Token};

pub enum SyntaxError {
    /// Unknown operator
    S001,
    /// Unknown symbol
    S002,
    /// Expected expression
    S003,
    /// Expected identifier
    S004,
    /// Missing semicolon
    S005,
    /// Missing closer (')', '}', etc)
    S006,
    /// Invalid assignment Target
    S007,
    /// Unterminated string
    S008,
    /// Expected block
    S009,
    /// Expected keyword
    S010 { keyword: String },
    /// Expected range operator
    S011,
}

pub enum TypeError {
    /// Unary operand mismatch
    T001 { operand: Value },
    /// Binary operands mismatch
    T002 { left: Value, right: Value },
    /// Range operand not a number
    T003 {
        from: Value,
        to: Value,
        step_by: Option<Value>,
    },
}

pub enum RuntimeError {
    /// Unknown identifier
    R001,
    /// Division by zero
    R002,
    /// Bad range
    R003 { from: f64, to: f64, step_by: f64 },
}

pub enum ErrorKind {
    Syntax(SyntaxError),
    Type(TypeError),
    Runtime(RuntimeError),
}

// Standard ono error type
pub struct Error {
    pub kind: ErrorKind,
    pub token: Token,
    pub file: Option<String>,
    pub line_src: Option<String>,
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
                let row_str = format!("{} | ", self.token.row + 1);
                let column_indicator = {
                    let spaces = std::iter::repeat(" ")
                        .take(row_str.len() + self.token.column - self.token.lexeme.len())
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
            Some(filename) => {
                let position = format!(" {}:{}", self.token.row + 1, self.token.column);
                format!("-> {}{}", filename, position).cyan().to_string()
            }
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
                SyntaxError::S001 => format!("expected operator, found '{}'", self.token.lexeme),
                SyntaxError::S002 => format!("unexpected symbol '{}'", self.token.lexeme),
                SyntaxError::S003 => format!("expected expression, found '{}'", self.token.lexeme),
                SyntaxError::S004 => format!("expected identifier, found '{}'", self.token.lexeme),
                SyntaxError::S005 => format!("expected ';', found '{}'", self.token.lexeme),
                SyntaxError::S006 => format!(
                    "expected '{}' closing this",
                    match self.token.lexeme.as_str() {
                        "{" => "}",
                        "(" => ")",
                        "\"" => "\"",
                        "[" => "]",
                        _ => panic!("Unhandled opener"),
                    }
                ),
                SyntaxError::S007 => format!("left-hand side is unassignable"),
                SyntaxError::S008 => format!("expected '\"' closing string starting here"),
                SyntaxError::S009 => format!("expected block, found '{}'", self.token.lexeme),
                SyntaxError::S010 { keyword } => format!(
                    "expected keyword '{}', found '{}'",
                    keyword, self.token.lexeme
                ),
                SyntaxError::S011 => format!(
                    "expected range operator '..', found '{}'",
                    self.token.lexeme
                ),
            },
            ErrorKind::Type(errno) => match errno {
                TypeError::T001 { operand } => {
                    format!(
                        "cannot perform '{} {}'",
                        self.token.lexeme,
                        operand.display_type()
                    )
                }
                TypeError::T002 { left, right } => format!(
                    "cannot perform '{} {} {}'",
                    left.display_type(),
                    self.token.lexeme,
                    right.display_type()
                ),
                TypeError::T003 { from, to, step_by } => format!(
                    "cannot make range with '{}..{}{}'",
                    from.display_type(),
                    if let Some(step_by) = step_by {
                        format!("{}..", step_by.display_type())
                    } else {
                        "".to_string()
                    },
                    to.display_type()
                ),
            },
            ErrorKind::Runtime(errno) => match errno {
                RuntimeError::R001 => format!("'{}' is not defined here", self.token.lexeme),
                RuntimeError::R002 => format!("division by zero"),
                RuntimeError::R003 { from, to, step_by } => format!(
                    "bad range running from '{}' to '{}' with step '{}'",
                    from, to, step_by
                ),
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
