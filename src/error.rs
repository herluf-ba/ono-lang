use std::fmt;

use crate::lexer::Token;

#[derive(Debug)]
pub enum ErrorKind {
    SyntaxError,
    RuntimeError,
}

// Standard ono error type
#[derive(Debug)]
pub struct Error {
    // Row and column are optional since they have no meaning in a REPL
    pub kind: ErrorKind,
    pub row: Option<usize>,
    pub column: Option<usize>,
    pub line_src: String,
    pub message: String,
}

impl Error {
    pub fn new(
        kind: ErrorKind,
        row: Option<usize>,
        column: Option<usize>,
        line_src: &str,
        message: &str,
    ) -> Self {
        Self {
            kind,
            column,
            row,
            line_src: line_src.to_string(),
            message: message.to_string(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let row_str = match self.row {
            Some(row) => format!("{} | ", row),
            None => String::new(),
        };

        let column_indicator = match self.column {
            Some(column) => {
                let spaces = std::iter::repeat(" ")
                    .take(row_str.len() + column - 1)
                    .collect::<String>();
                format!("\n{}^", spaces)
            }
            None => String::new(),
        };

        write!(
            f,
            "{:#?}: {}\n\n{}{}{}",
            self.kind, self.message, row_str, self.line_src, column_indicator
        )
    }
}

pub trait ErrorCollector {
    fn get_errors(&self) -> &Vec<Error>;

    fn has_errors(&self) -> bool {
        self.get_errors().len() > 0
    }

    fn report_errors(&self) {
        for error in self.get_errors() {
            println!("{}", error);
        }
    }
}

pub struct ErrorProducer<'a> {
    src_lines: Vec<&'a str>,
}

impl<'a> ErrorProducer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src_lines: src.split("\n").collect::<Vec<&'a str>>(),
        }
    }

    pub fn syntax_error_from_token(&self, token: &Token, message: &str) -> Error {
        Error::new(
            ErrorKind::SyntaxError,
            Some(token.row),
            Some(token.column),
            self.src_lines[token.row],
            message,
        )
    }

    pub fn runtime_error_from_token(&self, token: &Token, message: &str) -> Error {
        Error::new(
            ErrorKind::RuntimeError,
            Some(token.row),
            Some(token.column),
            self.src_lines[token.row],
            message,
        )
    }
}
