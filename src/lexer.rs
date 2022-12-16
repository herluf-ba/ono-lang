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
    IDENTIFIER(String),
    STRING(String),
    NUMBER(f64),

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

fn identifier_token_kind(c: &str) -> TokenKind {
    match c {
        "and" => TokenKind::AND,
        "class" => TokenKind::CLASS,
        "else" => TokenKind::ELSE,
        "false" => TokenKind::FALSE,
        "for" => TokenKind::FOR,
        "fun" => TokenKind::FUN,
        "if" => TokenKind::IF,
        "null" => TokenKind::NULL,
        "or" => TokenKind::OR,
        "print" => TokenKind::PRINT,
        "return" => TokenKind::RETURN,
        "super" => TokenKind::SUPER,
        "self" => TokenKind::SELF,
        "true" => TokenKind::TRUE,
        "let" => TokenKind::LET,
        "while" => TokenKind::WHILE,
        s => TokenKind::IDENTIFIER(s.to_string()),
    }
}

impl<'a> ErrorReporter for Lexer<'a> {
    fn add(&mut self, error: Error) {
        self.errors.push(error);
    }

    fn get_errors(&self) -> &Vec<Error> {
        &self.errors
    }
}

fn is_digit(c: &str) -> bool {
    match c {
        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => true,
        _ => false,
    }
}

fn is_alpha(c: &str) -> bool {
    (c >= "a" && c <= "z") || (c >= "A" && c <= "Z") || c == "_"
}

fn is_alpha_numeric(c: &str) -> bool {
    is_digit(c) || is_alpha(c)
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

    fn add_token(&mut self, kind: TokenKind) {
        let lexeme = self.graphemes[self.start..self.current].join("");
        self.tokens.push(Token {
            kind,
            lexeme,
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
        if self.current >= self.graphemes.len() {
            return "\0".to_string();
        }
        self.graphemes[self.current].to_string()
    }

    fn peekpeek(&self) -> String {
        if self.current + 1 >= self.graphemes.len() {
            return "\0".to_string();
        }
        self.graphemes[self.current + 1].to_string()
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c.as_str() {
            "(" => self.add_token(TokenKind::LEFTPAREN),
            ")" => self.add_token(TokenKind::RIGHTPAREN),
            "{" => self.add_token(TokenKind::LEFTBRACE),
            "}" => self.add_token(TokenKind::RIGHTBRACE),
            "," => self.add_token(TokenKind::COMMA),
            "." => self.add_token(TokenKind::DOT),
            "-" => self.add_token(TokenKind::MINUS),
            "+" => self.add_token(TokenKind::PLUS),
            ";" => self.add_token(TokenKind::SEMICOLON),
            "*" => self.add_token(TokenKind::STAR),
            "/" => self.add_token(TokenKind::SLASH),
            "!" => {
                if self.is_next("=") {
                    self.add_token(TokenKind::BANGEQUAL);
                } else {
                    self.add_token(TokenKind::BANG);
                }
            }
            "=" => {
                if self.is_next("=") {
                    self.add_token(TokenKind::EQUALEQUAL);
                } else {
                    self.add_token(TokenKind::EQUAL);
                }
            }
            "<" => {
                if self.is_next("=") {
                    self.add_token(TokenKind::LESSEQUAL);
                } else {
                    self.add_token(TokenKind::LESS);
                }
            }
            ">" => {
                if self.is_next("=") {
                    self.add_token(TokenKind::GREATEREQUAL);
                } else {
                    self.add_token(TokenKind::GREATER);
                }
            }

            // Comments
            "#" => loop {
                if self.advance() == "\n" {
                    break;
                }
            },

            "\"" => self.string(),
            s if is_digit(s) => self.number(),
            s if is_alpha(s) => self.identifier(),

            // Ignore whitespace
            "\n" | " " | "\t" => {}

            _ => self.add_error(&format!("Unexpected character '{}'", c)),
        }
    }

    fn string(&mut self) {
        while self.peek() != "\"" && !self.is_at_end() {
            self.advance();
        }

        if self.is_at_end() {
            self.add_error("Unterminated string");
            return;
        }

        // Eat up the closing "
        self.advance();

        let value = self.graphemes[(self.start + 1)..(self.current - 1)].join("");
        self.add_token(TokenKind::STRING(value))
    }

    fn number(&mut self) {
        // Read all digits
        while is_digit(&self.peek()) {
            self.advance();
        }

        // Read optional fraction too
        if self.peek() == "." && is_digit(&self.peekpeek()) {
            self.advance();

            while is_digit(&self.peek()) {
                self.advance();
            }
        }

        let value = self.graphemes[(self.start)..(self.current)].join("");
        self.add_token(TokenKind::NUMBER(value.parse::<f64>().unwrap()))
    }

    fn identifier(&mut self) {
        while is_alpha_numeric(&self.peek()) {
            self.advance();
        }

        let lexeme = self.graphemes[(self.start)..(self.current)].join("");
        self.add_token(identifier_token_kind(&lexeme))
    }
}
