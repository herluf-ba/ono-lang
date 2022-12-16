use std::fmt;

#[derive(Debug)]
pub enum ErrorKind {
    SyntaxError,
    TypeError,
    Error,
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

pub trait ErrorReporter {
    fn add(&mut self, error: Error);

    fn get_errors(&self) -> &Vec<Error>;

    fn report_errors(&self) {
        for error in self.get_errors() {
            println!("{}", error);
        }
    }

    fn is_ok(&self) -> bool {
        self.get_errors().len() == 0
    }
}

pub type Result<T> = std::result::Result<T, ()>;
