use crate::{
    error::{Error, SyntaxError},
    token::{Position, Token, TokenKind},
};
use unicode_segmentation::UnicodeSegmentation;

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
    tokens: Vec<Token>,
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

    /// Produces a `Vec` of tokens
    pub fn tokenize(&mut self, src: &str) -> Result<Vec<Token>, Vec<Error>> {
        // Split src into UTF-8 graphemes
        self.graphemes = UnicodeSegmentation::graphemes(src, true)
            .map(String::from)
            .collect::<Vec<String>>();

        let mut errors: Vec<Error> = Vec::new();
        while !self.is_at_end() {
            self.start = self.current;
            if let Err(error) = self.scan_token() {
                errors.push(error);
            }
        }

        // Add end of file
        self.add_token(TokenKind::EOF);

        // Reset internal state for a new tokenization
        let tokens = std::mem::replace(&mut self.tokens, Vec::new());
        self.graphemes = Vec::new();
        self.current = 0;
        self.start = 0;
        self.column = 0;
        self.line = 0;

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(tokens)
        }
    }

    fn current_position(&self) -> Position {
        Position::new(self.line, self.column)
    }

    fn add_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token::new(
            kind,
            self.current_position(),
            &self.graphemes[self.start..self.current].join(""),
        ));
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
            "\n" | " " | "\t" => {}
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
            ":" => self.add_token(TokenKind::COLON),
            "*" => self.add_token(TokenKind::STAR),
            "/" => self.add_token(TokenKind::SLASH),
            "?" => self.add_token(TokenKind::QUESTIONMARK),
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
            _ => {
                return Err(Error::syntax_error(
                    SyntaxError::S002,
                    Token::new(TokenKind::UNKNOWN, self.current_position(), &c),
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
                Token::new(
                    TokenKind::UNKNOWN,
                    Position::new(opening_row, opening_column),
                    "\"",
                ),
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
        let token = match lexeme.as_str() {
            "match" => TokenKind::MATCH,
            "trait" => TokenKind::TRAIT,
            "enum" => TokenKind::ENUM,
            "obj" => TokenKind::OBJ,
            "some" => TokenKind::SOME,
            "none" => TokenKind::NONE,
            "has" => TokenKind::HAS,
            "and" => TokenKind::AND,
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
            "self" => TokenKind::SELF,
            "true" => TokenKind::TRUE,
            "let" => TokenKind::LET,
            "while" => TokenKind::WHILE,
            s => TokenKind::IDENTIFIER(s.to_string()),
        };

        self.add_token(token)
    }
}
