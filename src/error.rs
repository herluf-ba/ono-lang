use colored::Colorize;
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
    pub kind: ErrorKind,
    pub file: Option<String>,
    pub row: Option<usize>,
    pub column: Option<usize>,
    pub len: usize,
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
            len: 1,
            message: message.to_string(),
            file: None,
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
            file: None,
            column: Some(token.column),
            len: token.lexeme.len(),
            row: Some(token.row),
            message: message.to_string(),
            line_src: None,
        }
    }

    pub fn add_src(&mut self, line_src: &str) {
        self.line_src = Some(line_src.to_string())
    }

    pub fn add_filename(&mut self, filename: &str) {
        self.file = Some(filename.to_string())
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
                    .take(row_str.len() + column - self.len)
                    .collect::<String>();
                let arrows = std::iter::repeat("^").take(self.len).collect::<String>();
                format!("\n{}{}", spaces, arrows)
            }
            None => String::new(),
        };

        let line_src = match &self.line_src {
            Some(line_src) => line_src,
            None => "",
        };

        let file_name = match &self.file {
            Some(file) => {
                let position = match self.column {
                    Some(column) => match self.row {
                        Some(row) => format!(" {}:{}", row + 1, column),
                        None => "".to_string(),
                    },
                    None => "".to_string(),
                };
                format!("-> {}{}\n", file, position).to_string()
            }
            None => "".to_string(),
        };

        write!(
            f,
            "\n{}: {}\n{}{}{}{}",
            format!("{:#?}", self.kind).bright_red().bold(),
            self.message.bold(),
            file_name.cyan(),
            row_str,
            line_src,
            column_indicator.bright_red().bold()
        )
    }
}
