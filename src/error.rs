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
    pub line_src: Option<String>,
    pub message: String,
}

impl Error {
    pub fn new(
        kind: ErrorKind,
        row: Option<usize>,
        column: Option<usize>,
        line_src: Option<&str>,
        message: &str,
    ) -> Self {
        Self {
            kind,
            column,
            row,
            message: message.to_string(),
            line_src: if let Some(line_src) = line_src {
                Some(line_src.to_string())
            } else {
                None
            },
        }
    }

    pub fn from_token(token: &Token, kind: ErrorKind, message: &str) -> Self {
        Self {
            kind,
            column: Some(token.column),
            row: Some(token.row),
            message: message.to_string(),
            line_src: None,
        }
    }

    pub fn add_src(&mut self, line_src: &str) {
        self.line_src = Some(line_src.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let row_str = match self.row {
            Some(row) => format!("{} | ", row + 1),
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

        let line_src = match &self.line_src {
            Some(line_src) => line_src,
            None => "",
        };

        write!(
            f,
            "{:#?}: {}\n{}{}{}\n",
            self.kind, self.message, row_str, line_src, column_indicator
        )
    }
}
