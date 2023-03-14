use crate::{
    error::{Error, SyntaxError},
    types::{Token, TokenKind},
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
    /// current line in src
    line: usize,
    /// column number of current in src.
    /// Note that this is not the column start of the lexeme but the end
    column_end: usize,
}

impl Lexer {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            graphemes: Vec::new(),
            start: 0,
            current: 0,
            line: 0,
            column_end: 0,
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
        self.tokens
            .push(Token::new(TokenKind::EOF, self.line + 1, 0, "\n"));

        // Reset internal state for a new tokenization
        let tokens = std::mem::replace(&mut self.tokens, Vec::new());
        self.graphemes = Vec::new();
        self.current = 0;
        self.start = 0;
        self.column_end = 0;
        self.line = 0;

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(tokens)
        }
    }

    fn add_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token::new(
            kind,
            self.line,
            self.column_end - (self.current - self.start),
            &self.graphemes[self.start..self.current].join(""),
        ));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.graphemes.len()
    }

    fn advance(&mut self) -> String {
        let grapheme = self.peek();

        self.current += 1;
        self.column_end += 1;
        if grapheme == "\n" {
            self.line += 1;
            self.column_end = 0;
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
            "[" => self.add_token(TokenKind::LEFTBRACKET),
            "]" => self.add_token(TokenKind::RIGHTBRACKET),
            "{" => self.add_token(TokenKind::LEFTBRACE),
            "}" => self.add_token(TokenKind::RIGHTBRACE),
            "-" => self.add_token(TokenKind::MINUS),
            "+" => self.add_token(TokenKind::PLUS),
            "*" => self.add_token(TokenKind::STAR),
            "/" => self.add_token(TokenKind::SLASH),
            "," => self.add_token(TokenKind::COMMA),
            ":" => self.add_token(TokenKind::COLON),
            ";" => self.add_token(TokenKind::SEMICOLON),
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
                    Token::new(
                        TokenKind::UNKNOWN,
                        self.line,
                        self.column_end - (self.current - self.start),
                        &c,
                    ),
                ));
            }
        };

        Ok(())
    }

    fn string(&mut self) -> Result<(), Error> {
        let opening_row = self.line;
        let opening_column = self.column_end - 1;

        while self.peek() != "\"" && !self.is_at_end() {
            self.advance();
        }

        if self.is_at_end() {
            return Err(Error::syntax_error(
                SyntaxError::S002,
                Token::new(TokenKind::UNKNOWN, opening_row, opening_column, "\""),
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
            "let" => TokenKind::LET,
            "string" => TokenKind::STRINGKW,
            "number" => TokenKind::NUMBERKW,
            "bool" => TokenKind::BOOL,
            identifier => TokenKind::IDENTIFIER(identifier.to_string()),
        };

        self.add_token(token)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use TokenKind::*;

    #[test]
    fn tokenizes() -> Result<(), Vec<Error>> {
        let src = r#"( , ) - + / * ! != = == > >= < <= true false and or "test" 123 123.45"#;
        let target = vec![
            Token::new(LEFTPAREN, 0, 0, "("),
            Token::new(COMMA, 0, 2, ","),
            Token::new(RIGHTPAREN, 0, 4, ")"),
            Token::new(MINUS, 0, 6, "-"),
            Token::new(PLUS, 0, 8, "+"),
            Token::new(SLASH, 0, 10, "/"),
            Token::new(STAR, 0, 12, "*"),
            Token::new(BANG, 0, 14, "!"),
            Token::new(BANGEQUAL, 0, 16, "!="),
            Token::new(EQUAL, 0, 19, "="),
            Token::new(EQUALEQUAL, 0, 21, "=="),
            Token::new(GREATER, 0, 24, ">"),
            Token::new(GREATEREQUAL, 0, 26, ">="),
            Token::new(LESS, 0, 29, "<"),
            Token::new(LESSEQUAL, 0, 31, "<="),
            Token::new(TRUE, 0, 34, "true"),
            Token::new(FALSE, 0, 39, "false"),
            Token::new(AND, 0, 45, "and"),
            Token::new(OR, 0, 49, "or"),
            Token::new(STRING("test".to_string()), 0, 52, "\"test\""),
            Token::new(NUMBER(123.0), 0, 59, "123"),
            Token::new(NUMBER(123.45), 0, 63, "123.45"),
            Token::new(EOF, 1, 0, "\n"),
        ];

        assert_eq!(Lexer::new().tokenize(src)?, target);
        Ok(())
    }

    #[test]
    fn handles_comments() -> Result<(), Vec<Error>> {
        let src = r###"
            # This is a comment
            1 + 2
            # Comment with '#'
            1 + 2 # this is an inline comment so that we dont also + 3
        "###;

        let target = vec![
            Token::new(NUMBER(1.0), 2, 12, "1"),
            Token::new(PLUS, 2, 14, "+"),
            Token::new(NUMBER(2.0), 2, 16, "2"),
            Token::new(NUMBER(1.0), 4, 12, "1"),
            Token::new(PLUS, 4, 14, "+"),
            Token::new(NUMBER(2.0), 4, 16, "2"),
            Token::new(EOF, 6, 0, "\n"),
        ];
        assert_eq!(Lexer::new().tokenize(src)?, target);
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
                Token::new(TokenKind::UNKNOWN, 0, 0, src)
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
                Token::new(TokenKind::UNKNOWN, 0, 0, "\"")
            )])
        );
        Ok(())
    }
}
