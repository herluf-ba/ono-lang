use std::fmt::Display;

use unicode_segmentation::UnicodeSegmentation;

pub enum TokenKind {
    // Single-character tokens.
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
    // One or two character tokens.
    BANG,
    BANGEQUAL,
    EQUAL,
    EQUALEQUAL,
    GREATER,
    GREATEREQUAL,
    LESS,
    LESSEQUAL,
    // Literals.
    IDENTIFIER,
    STRING,
    NUMBER,
    // Keywords.
    AND,
    CLASS,
    ELSE,
    FALSE,
    FOR,
    FUN,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    ERROR,
    EOF,
}

pub struct Token<'a> {
    kind: TokenKind,
    lexeme: &'a str,
    line: usize,
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lexeme)
    }
}

fn is_digit(c: &str) -> bool {
    matches!(c, "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
}

fn is_alpha(c: &str) -> bool {
    (c >= "a" && c <= "z") || (c >= "A" && c <= "Z") || c == "_"
}

fn is_alpha_numeric(c: &str) -> bool {
    is_digit(c) || is_alpha(c)
}

pub struct Scanner<'a> {
    /// Graphemes being tokenized
    graphemes: Vec<&'a str>,
    /// Start of the current token being read
    start: usize,
    /// Current read head
    current: usize,
    /// current line in src
    line: usize,
}

impl<'a> std::iter::Iterator for Scanner<'a> {
    type Item = Result<Token<'a>, String>;

    fn next(&mut self) -> Option<Result<Token<'a>, String>> {
        if self.is_at_end() {
            None
        } else {
            Some(self.next_token())
        }
    }
}

impl<'a> Scanner<'a> {
    pub fn new(src: &'a str) -> Self {
        // Split src into UTF-8 graphemes
        let graphemes = UnicodeSegmentation::graphemes(src, true).collect::<Vec<&str>>();
        Self {
            graphemes,
            start: 0,
            current: 0,
            line: 0,
        }
    }

    fn next_token(&mut self) -> Result<Token, String> {
        if self.is_at_end() {
            return Ok(self.make_token(TokenKind::EOF));
        }

        self.scan_token()
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            line: self.line,
            lexeme: &self.graphemes[self.start..self.current].join(""),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.graphemes.len()
    }

    fn advance(&mut self) -> &str {
        let grapheme = self.peek();

        if grapheme == "\n" {
            self.line += 1;
        }

        self.current += 1;
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

    fn peek(&self) -> &str {
        if self.current >= self.graphemes.len() {
            return "\0";
        }
        self.graphemes[self.current]
    }

    fn peekpeek(&self) -> String {
        if self.current + 1 >= self.graphemes.len() {
            return "\0".to_string();
        }
        self.graphemes[self.current + 1].to_string()
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                "\n" => {
                    self.line += 1;
                    self.advance();
                }
                c if c.chars().all(char::is_whitespace) => {
                    self.advance();
                }
                "#" => {
                    while self.peek() != "\n" && !self.is_at_end() {
                        self.advance();
                    }
                }
                _ => {
                    break;
                }
            }
        }
    }

    fn scan_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace_and_comments();

        match self.advance() {
            "(" => Ok(self.make_token(TokenKind::LEFTPAREN)),
            ")" => Ok(self.make_token(TokenKind::RIGHTPAREN)),
            "{" => Ok(self.make_token(TokenKind::LEFTBRACE)),
            "}" => Ok(self.make_token(TokenKind::RIGHTBRACE)),
            "-" => Ok(self.make_token(TokenKind::MINUS)),
            "+" => Ok(self.make_token(TokenKind::PLUS)),
            "*" => Ok(self.make_token(TokenKind::STAR)),
            "/" => Ok(self.make_token(TokenKind::SLASH)),
            "," => Ok(self.make_token(TokenKind::COMMA)),
            ";" => Ok(self.make_token(TokenKind::SEMICOLON)),
            "!" => {
                if self.is_next("=") {
                    Ok(self.make_token(TokenKind::BANGEQUAL))
                } else {
                    Ok(self.make_token(TokenKind::BANG))
                }
            }
            "=" => {
                if self.is_next("=") {
                    Ok(self.make_token(TokenKind::EQUALEQUAL))
                } else {
                    Ok(self.make_token(TokenKind::EQUAL))
                }
            }
            "<" => {
                if self.is_next("=") {
                    Ok(self.make_token(TokenKind::LESSEQUAL))
                } else {
                    Ok(self.make_token(TokenKind::LESS))
                }
            }
            ">" => {
                if self.is_next("=") {
                    Ok(self.make_token(TokenKind::GREATEREQUAL))
                } else {
                    Ok(self.make_token(TokenKind::GREATER))
                }
            }
            "\"" => self.string(),
            s if is_digit(s) => self.number(),
            s if is_alpha(s) => self.keyword(),
            s => Err(format!("unexpected symbol '{}'", s)),
        }
    }

    fn string(&mut self) -> Result<Token, String> {
        let opening_row = self.line;
        let opening_column = self.current;

        while self.peek() != "\"" && !self.is_at_end() {
            if self.peek() == "\n" {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(format!(
                "unterminated string at <{}, {}>",
                opening_row, opening_column
            ));
        }

        // Eat up the closing "
        self.advance();

        Ok(self.make_token(TokenKind::STRING))
    }

    fn number(&mut self) -> Result<Token, String> {
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

        Ok(self.make_token(TokenKind::NUMBER))
    }

    fn keyword(&mut self) -> Result<Token, String> {
        while is_alpha_numeric(&self.peek()) {
            self.advance();
        }

        let lexeme = self.graphemes[(self.start)..(self.current)].join("");
        let token = match lexeme.as_str() {
            "and" => TokenKind::AND,
            "class" => TokenKind::CLASS,
            "for" => TokenKind::FOR,
            "fun" => TokenKind::FUN,
            "nil" => TokenKind::NIL,
            "print" => TokenKind::PRINT,
            "return" => TokenKind::RETURN,
            "super" => TokenKind::SUPER,
            "this" => TokenKind::THIS,
            "var" => TokenKind::VAR,
            "while" => TokenKind::WHILE,
            "false" => TokenKind::FALSE,
            "or" => TokenKind::OR,
            "true" => TokenKind::TRUE,
            "if" => TokenKind::IF,
            "else" => TokenKind::ELSE,
            _ => TokenKind::IDENTIFIER,
        };

        Ok(self.make_token(token))
    }
}
