use crate::{
    error::{Error, SyntaxError},
    types::{Position, Token, TokenKind},
};
use unicode_segmentation::UnicodeSegmentation;

fn is_digit(c: &str) -> bool {
    matches!(c, "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
}

fn is_alpha(c: &str) -> bool {
    (c >= "a" && c <= "z") || (c >= "A" && c <= "Z") || c == "_"
}

fn is_alpha_numeric(c: &str) -> bool {
    is_digit(c) || is_alpha(c)
}

pub struct Lexer {
    /// Contains tokens produced so far
    tokens: Vec<Token>,
    /// Graphemes being tokenized
    graphemes: Vec<String>,
    /// Start of the current token being read
    start: usize,
    /// Current read head
    current: usize,
    /// Src position
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

    /// Produces a `Vec` of tokens by splitting into UTF-8 graphemes and doing a single pass
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
            "-" => self.add_token(TokenKind::MINUS),
            "+" => self.add_token(TokenKind::PLUS),
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
            s if is_alpha(s) => self.keyword(),
            _ => {
                return Err(Error::syntax_error(
                    SyntaxError::S001,
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
                SyntaxError::S002,
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

    fn keyword(&mut self) {
        while is_alpha_numeric(&self.peek()) {
            self.advance();
        }

        let lexeme = self.graphemes[(self.start)..(self.current)].join("");
        let token = match lexeme.as_str() {
            "and" => TokenKind::AND,
            "false" => TokenKind::FALSE,
            "or" => TokenKind::OR,
            "true" => TokenKind::TRUE,
            _ => todo!("identifier"),
        };

        self.add_token(token)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use TokenKind::*;

    #[test]
    fn tokenizes() -> Result<(), Vec<Error>> {
        let mut lexer = Lexer::new();
        let src = r#"( ) - + / * ! != = == > >= < <= true false and or "test" 123 123.45"#;
        let tokens = lexer.tokenize(src)?;

        let target = vec![
            Token {
                kind: LEFTPAREN,
                lexeme: "(".to_string(),
                position: Position { line: 0, column: 1 },
            },
            Token {
                kind: RIGHTPAREN,
                lexeme: ")".to_string(),
                position: Position { line: 0, column: 3 },
            },
            Token {
                kind: MINUS,
                lexeme: "-".to_string(),
                position: Position { line: 0, column: 5 },
            },
            Token {
                kind: PLUS,
                lexeme: "+".to_string(),
                position: Position { line: 0, column: 7 },
            },
            Token {
                kind: SLASH,
                lexeme: "/".to_string(),
                position: Position { line: 0, column: 9 },
            },
            Token {
                kind: STAR,
                lexeme: "*".to_string(),
                position: Position {
                    line: 0,
                    column: 11,
                },
            },
            Token {
                kind: BANG,
                lexeme: "!".to_string(),
                position: Position {
                    line: 0,
                    column: 13,
                },
            },
            Token {
                kind: BANGEQUAL,
                lexeme: "!=".to_string(),
                position: Position {
                    line: 0,
                    column: 16,
                },
            },
            Token {
                kind: EQUAL,
                lexeme: "=".to_string(),
                position: Position {
                    line: 0,
                    column: 18,
                },
            },
            Token {
                kind: EQUALEQUAL,
                lexeme: "==".to_string(),
                position: Position {
                    line: 0,
                    column: 21,
                },
            },
            Token {
                kind: GREATER,
                lexeme: ">".to_string(),
                position: Position {
                    line: 0,
                    column: 23,
                },
            },
            Token {
                kind: GREATEREQUAL,
                lexeme: ">=".to_string(),
                position: Position {
                    line: 0,
                    column: 26,
                },
            },
            Token {
                kind: LESS,
                lexeme: "<".to_string(),
                position: Position {
                    line: 0,
                    column: 28,
                },
            },
            Token {
                kind: LESSEQUAL,
                lexeme: "<=".to_string(),
                position: Position {
                    line: 0,
                    column: 31,
                },
            },
            Token {
                kind: TRUE,
                lexeme: "true".to_string(),
                position: Position {
                    line: 0,
                    column: 36,
                },
            },
            Token {
                kind: FALSE,
                lexeme: "false".to_string(),
                position: Position {
                    line: 0,
                    column: 42,
                },
            },
            Token {
                kind: AND,
                lexeme: "and".to_string(),
                position: Position {
                    line: 0,
                    column: 46,
                },
            },
            Token {
                kind: OR,
                lexeme: "or".to_string(),
                position: Position {
                    line: 0,
                    column: 49,
                },
            },
            Token {
                kind: STRING("test".to_string()),
                lexeme: "\"test\"".to_string(),
                position: Position {
                    line: 0,
                    column: 56,
                },
            },
            Token {
                kind: NUMBER(123.0),
                lexeme: "123".to_string(),
                position: Position {
                    line: 0,
                    column: 60,
                },
            },
            Token {
                kind: NUMBER(123.45),
                lexeme: "123.45".to_string(),
                position: Position {
                    line: 0,
                    column: 67,
                },
            },
            Token {
                kind: EOF,
                lexeme: "123.45".to_string(),
                position: Position {
                    line: 0,
                    column: 67,
                },
            },
        ];

        assert_eq!(tokens, target);
        Ok(())
    }

    #[test]
    fn handles_comments() -> Result<(), Vec<Error>> {
        let mut lexer = Lexer::new();
        let src = r###"
            # This is a comment
            1 + 2
            # Comment with '#'
            1 + 2 # this is an inline comment so that we dont also + 3
        "###;

        let tokens = lexer.tokenize(src)?;
        let target = vec![
            Token {
                kind: NUMBER(1.0),
                lexeme: "1".to_string(),
                position: Position {
                    line: 2,
                    column: 13,
                },
            },
            Token {
                kind: PLUS,
                lexeme: "+".to_string(),
                position: Position {
                    line: 2,
                    column: 15,
                },
            },
            Token {
                kind: NUMBER(2.0),
                lexeme: "2".to_string(),
                position: Position {
                    line: 2,
                    column: 17,
                },
            },
            Token {
                kind: NUMBER(1.0),
                lexeme: "1".to_string(),
                position: Position {
                    line: 4,
                    column: 13,
                },
            },
            Token {
                kind: PLUS,
                lexeme: "+".to_string(),
                position: Position {
                    line: 4,
                    column: 15,
                },
            },
            Token {
                kind: NUMBER(2.0),
                lexeme: "2".to_string(),
                position: Position {
                    line: 4,
                    column: 17,
                },
            },
            Token {
                kind: EOF,
                lexeme: " ".to_string(),
                position: Position { line: 5, column: 8 },
            },
        ];
        assert_eq!(tokens, target);
        Ok(())
    }

    #[test]
    fn errors_on_unexpected_symbol() -> Result<(), Vec<Error>> {
        let mut lexer = Lexer::new();
        let src = "💩";

        assert_eq!(
            lexer.tokenize(src),
            Err(vec![Error::syntax_error(
                SyntaxError::S001,
                Token::new(TokenKind::UNKNOWN, Position::new(0, 1), src)
            )])
        );
        Ok(())
    }

    #[test]
    fn errors_on_unterminated_string() -> Result<(), Vec<Error>> {
        let mut lexer = Lexer::new();
        let src = "\"whoops forgot to terminate this one";

        assert_eq!(
            lexer.tokenize(src),
            Err(vec![Error::syntax_error(
                SyntaxError::S002,
                Token::new(TokenKind::UNKNOWN, Position::new(0, 1), "\"")
            )])
        );
        Ok(())
    }
}
