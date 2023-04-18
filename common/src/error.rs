use std::ops::Range;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

// TODO: This whole file assumes that there is only a single source file
// and that therefore `()` can be used as a FileId for codespan_reporting
type FileId = ();

/// A common trait for all ono errors
pub trait Report {
    fn report(&self) -> Diagnostic<FileId>;
}

#[derive(Debug, PartialEq, Clone)]
pub enum SyntaxError {
    UnexpectedSymbol { symbol: Range<usize> },
    BadNumberLiteral { literal: Range<usize> },
    UnterminatedString { opening: Range<usize> },
}

impl Default for SyntaxError {
    fn default() -> Self {
        Self::UnexpectedSymbol { symbol: 0..0 }
    }
}

impl Report for SyntaxError {
    fn report(&self) -> Diagnostic<FileId> {
        match self {
            SyntaxError::UnexpectedSymbol { symbol } => Diagnostic::error()
                .with_code("S001")
                .with_message("unexpected symbol")
                .with_labels(vec![Label::primary((), symbol.clone())]),
            SyntaxError::BadNumberLiteral { literal } => Diagnostic::error()
                .with_code("S002")
                .with_message("cannot parse number literal")
                .with_labels(vec![Label::primary((), literal.clone())]),
            SyntaxError::UnterminatedString { opening } => Diagnostic::error()
                .with_code("S003")
                .with_message("unterminated string")
                .with_labels(vec![
                    Label::primary((), opening.clone()).with_message("starts here")
                ]),
        }
    }
}

pub fn report_errors<T>(
    file_name: &str,
    source: &str,
    errors: Vec<impl Report>,
) -> Result<(), codespan_reporting::files::Error> {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();
    let file = SimpleFile::new(file_name, source);

    for err in errors {
        let diagnostic = err.report();
        term::emit(&mut writer.lock(), &config, &file, &diagnostic)?;
    }

    Ok(())
}
