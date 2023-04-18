use std::ops::Range;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

// TODO: This whole file assumes that there is only a single source file
// and that therefore `()` can be used as a FileId for codespan_reporting
type FileId = ();

/// An error item spanning some source code
#[derive(Debug)]
pub struct Item {
    span: Range<usize>,
    message: Option<String>,
}

impl Item {
    pub fn new(span: Range<usize>, message: Option<impl Into<String>>) -> Self {
        Self {
            span, 
            message: message.map(|message| message.into()),
        }
    }
}

impl Into<Label<FileId>> for &Item {
    fn into(self) -> Label<FileId> {
        let mut label = Label::primary((), self.span.clone());
        if let Some(message) = &self.message {
            label = label.with_message(message);
        }
        label
    }
}

/// A common trait for all ono errors
pub trait Report {
    fn report(&self) -> Diagnostic<FileId>;
}

#[derive(Debug, PartialEq, Clone)]
pub enum SyntaxError {
    UnexpectedSymbol { symbol: Range<usize> },
    BadNumberLiteral { literal: Range<usize> },
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
