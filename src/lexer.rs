use std::fmt::Display;

use unicode_segmentation::UnicodeSegmentation;

use crate::error::{Error, SyntaxError};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TokenKind {
    // Single-character tokens
    LEFTPAREN,
    RIGHTPAREN,
    LEFTBRACE,
    RIGHTBRACE,
    COMMA,
    DOT,
    DOTDOT,
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

    // Literals()
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
    IN,
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

impl TokenKind {
    pub fn is_same(&self, other: &TokenKind) -> bool {
        //Compare only by variant
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub row: usize,
    pub column: usize,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lexeme)
    }
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
        "in" => TokenKind::IN,
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

fn is_digit(c: &str) -> bool {
    match c {
        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => true,
        _ => false,
    }
}

fn is_alpha(c: &str) -> bool {
    (c >= "a" && c <= "z") || (c >= "A" && c <= "Z") || c == "_"
}

fn is_alpha_numeric(c: &str) -> bool {
    is_digit(c) || is_alpha(c)
}

pub struct Lexer {
    pub tokens: Vec<Token>,
    graphemes: Vec<String>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            graphemes: Vec::new(),
            start: 0,
            current: 0,
            line: 0,
            column: 0,
        }
    }

    pub fn reset(&mut self) {
        self.tokens = Vec::new();
        self.graphemes = Vec::new();
        self.current = 0;
        self.start = 0;
        self.column = 0;
    }

    pub fn tokenize(&mut self, src: &str) -> Result<Vec<Token>, Vec<Error>> {
        let mut errors: Vec<Error> = Vec::new();
        self.graphemes = UnicodeSegmentation::graphemes(src, true)
            .map(String::from)
            .collect::<Vec<String>>();
        while !self.is_at_end() {
            self.start = self.current;
            if let Err(error) = self.scan_token() {
                errors.push(error);
            }
        }
        self.add_token(TokenKind::EOF);
        let tokens = self.tokens.clone();
        self.reset();
        self.line += 1;
        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(tokens)
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

    fn scan_token(&mut self) -> Result<(), Error> {
        let c = self.advance();

        match c.as_str() {
            "(" => self.add_token(TokenKind::LEFTPAREN),
            ")" => self.add_token(TokenKind::RIGHTPAREN),
            "{" => self.add_token(TokenKind::LEFTBRACE),
            "}" => self.add_token(TokenKind::RIGHTBRACE),
            "," => self.add_token(TokenKind::COMMA),
            "." => {
                if self.is_next(".") {
                    self.add_token(TokenKind::DOTDOT);
                } else {
                    self.add_token(TokenKind::DOT);
                }
            }
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

            "\"" => {
                if let Err(error) = self.string() {
                    return Err(error);
                }
            }
            s if is_digit(s) => self.number(),
            s if is_alpha(s) => self.identifier(),

            // Ignore whitespace
            "\n" | " " | "\t" => {}

            _ => {
                return Err(Error::syntax_error(
                    SyntaxError::S002,
                    Token {
                        kind: TokenKind::NULL, // This is wrong but the kind is unknown here
                        lexeme: c,
                        row: self.line,
                        column: self.column,
                    },
                ));
            }
        };
        Ok(())
    }

    fn string(&mut self) -> Result<(), Error> {
        let opening_row = self.line;
        let opening_column = self.column;

        while self.peek() != "\"" && !self.is_at_end() {
            self.advance();
        }

        if self.is_at_end() {
            return Err(Error::syntax_error(
                SyntaxError::S008,
                Token {
                    kind: TokenKind::NULL, // This is wrong but theres no '"' TokenKind
                    row: opening_row,
                    column: opening_column,
                    lexeme: self.peek(),
                },
            ));
        }

        // Eat up the closing "
        self.advance();

        let value = self.graphemes[(self.start + 1)..(self.current - 1)].join("");
        self.add_token(TokenKind::STRING(value));
        Ok(())
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
