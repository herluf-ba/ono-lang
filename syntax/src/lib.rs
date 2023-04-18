use common::error::SyntaxError;
use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")] // Ignore whitespace
#[logos(error=SyntaxError)]
pub enum Token {
    // Single character tokens
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("-")]
    Minus,
    #[token("+")]
    Plus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token(";")]
    Semicolon,
    #[token("!")]
    Bang,
    #[token("<")]
    Less,
    #[token("=")]
    Equal,
    #[token(">")]
    Greater,

    // Double character tokens
    #[token(">=")]
    LessEqual,
    #[token("<=")]
    GreaterEqual,
    #[token("!=")]
    BangEqual,
    #[token("==")]
    EqualEqual,

    // Keywords
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("fun")]
    Fun,
    #[token("return")]
    Return,
    #[token("var")]
    Var,
    #[token("class")]
    Class,
    #[token("this")]
    This,
    #[token("super")]
    Super,
    #[token("If")]
    If,
    #[token("else")]
    Else,
    #[token("false")]
    False,
    #[token("true")]
    True,
    #[token("for")]
    For,
    #[token("while")]
    While,
    #[token("nil")]
    Nil,
    #[token("print")]
    Print,

    // Literals
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", lex_identifier)]
    Identifier(String),
    #[regex(r#""[^"]*""#, lex_string)]
    String(String),
    #[regex(r#"[0-9]+(\.[0-9]+)?"#, lex_number)]
    Number(f64),

    // Comments
    #[regex(r"//.*", lex_comment)]
    Comment(String),
}

fn lex_string(lexer: &mut logos::Lexer<Token>) -> String {
    let slice = lexer.slice();
    slice[1..slice.len() - 1].to_string()
}

fn lex_identifier(lexer: &mut logos::Lexer<Token>) -> String {
    let slice = lexer.slice();
    slice.to_string()
}

fn lex_number(lexer: &mut logos::Lexer<Token>) -> Result<f64, SyntaxError> {
    let slice = lexer.slice();
    match slice.parse::<f64>() {
        Ok(v) => Ok(v),
        Err(_) => Err(SyntaxError::BadNumberLiteral {
            literal: lexer.span(),
        }),
    }
}

fn lex_comment(lexer: &mut logos::Lexer<Token>) -> String {
    let slice = lexer.slice();
    slice[2..slice.len()].to_string()
}

pub struct Scanner<'a> {
    inner: logos::Lexer<'a, Token>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            inner: Token::lexer(source),
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token, SyntaxError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            None => None,
            Some(Err(_)) => Some(Err(SyntaxError::UnexpectedSymbol {
                symbol: self.inner.span(),
            })),
            Some(t) => Some(t),
        }
    }
}
