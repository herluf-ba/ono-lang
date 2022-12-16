use crate::error::*;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub enum TokenKind {
    // Single-character tokens
    LEFTPAREN,
    RIGHTPAREN,
    LEFTBRACE,
    RIGHTBRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // Comparators
    BANG,
    BANGEQUAL,
    EQUAL,
    EQUALEQUAL,
    GREATER,
    GREATEREQUAL,
    LESS,
    LESSEQUAL,

    // Literals
    IDENTIFIER,
    STRING,
    NUMBER,

    // KEYWORDS
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NULL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    SELF,
    TRUE,
    LET,
    WHILE,

    // Internal
    EOF,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub row: usize,
    pub column: usize,
}

pub struct Lexer<'a> {
    pub tokens: Vec<Token>,
    pub errors: Vec<Error>,
    graphemes: Vec<&'a str>,
    lines: Vec<&'a str>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
}

impl<'a> ErrorReporter for Lexer<'a> {
    fn add(&mut self, error: Error) {
        self.errors.push(error);
    }

    fn get_errors(&self) -> &Vec<Error> {
        &self.errors
    }
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            graphemes: UnicodeSegmentation::graphemes(src, true).collect::<Vec<&'a str>>(),
            tokens: Vec::new(),
            errors: Vec::new(),
            lines: src.split("\n").collect::<Vec<&'a str>>(),
            start: 0,
            current: 0,
            line: 0,
            column: 0,
        }
    }

    pub fn tokenize(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
    }

    fn add_token(&mut self, lexeme: String, kind: TokenKind) {
        self.tokens.push(Token {
            kind,
            lexeme: lexeme.to_string(),
            row: self.line,
            column: self.column,
        });
    }

    fn add_error(&mut self, message: &str) {
        self.errors.push(Error::new(
            ErrorKind::SyntaxError,
            Some(self.line + 1),
            Some(self.column),
            self.lines[self.line],
            message,
        ))
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.graphemes.len()
    }

    fn advance(&mut self) -> String {
        let grapheme = self.peek();

        self.current += 1;
        self.column += 1;
        if grapheme == "\n" {
            self.line += 1;
            self.column = 0;
        }

        grapheme
    }

    fn is_next(&mut self, expected: &str) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.graphemes[self.current] != expected {
            return false;
        }

        // Increment current if we found the expected grapheme
        self.advance();
        true
    }

    fn peek(&self) -> String {
        self.graphemes[self.current].to_string()
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c.as_str() {
            "(" => self.add_token(c, TokenKind::LEFTPAREN),
            ")" => self.add_token(c, TokenKind::RIGHTPAREN),
            "{" => self.add_token(c, TokenKind::LEFTBRACE),
            "}" => self.add_token(c, TokenKind::RIGHTBRACE),
            "," => self.add_token(c, TokenKind::COMMA),
            "." => self.add_token(c, TokenKind::DOT),
            "-" => self.add_token(c, TokenKind::MINUS),
            "+" => self.add_token(c, TokenKind::PLUS),
            ";" => self.add_token(c, TokenKind::SEMICOLON),
            "*" => self.add_token(c, TokenKind::STAR),
            "/" => self.add_token(c, TokenKind::SLASH),
            "!" => {
                if self.is_next("=") {
                    self.add_token(c, TokenKind::BANGEQUAL);
                } else {
                    self.add_token(c, TokenKind::BANG);
                }
            }
            "=" => {
                if self.is_next("=") {
                    self.add_token(c, TokenKind::EQUALEQUAL);
                } else {
                    self.add_token(c, TokenKind::EQUAL);
                }
            }
            "<" => {
                if self.is_next("=") {
                    self.add_token(c, TokenKind::LESSEQUAL);
                } else {
                    self.add_token(c, TokenKind::EQUAL);
                }
            }
            ">" => {
                if self.is_next("=") {
                    self.add_token(c, TokenKind::GREATEREQUAL);
                } else {
                    self.add_token(c, TokenKind::EQUAL);
                }
            }

            // Comments
            "#" => loop {
                if self.advance() == "\n" {
                    break;
                }
            },

            // Ignore whitespace
            "\n" | " " | "\t" => {}

            _ => self.add_error(&format!("Unexpected character '{}'", c)),
        }
    }
}
