use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    LEFTPAREN,
    RIGHTPAREN,
    LEFTBRACKET,
    RIGHTBRACKET,
    LEFTBRACE,
    RIGHTBRACE,

    COMMA,
    EQUAL,
    COLON,
    SEMICOLON,

    // Keywords
    LET,
    IF,
    ELSE,
    STRINGKW,
    NUMBERKW,
    BOOL,

    // Math operators
    MINUS,
    PLUS,
    SLASH,
    STAR,

    // Logical operators
    BANGEQUAL,
    EQUALEQUAL,
    GREATER,
    GREATEREQUAL,
    LESS,
    LESSEQUAL,
    BANG,
    AND,
    OR,

    // Literals
    STRING(String),
    NUMBER(f64),
    TRUE,
    FALSE,
    IDENTIFIER(String),

    // Internal
    EOF,
    UNKNOWN,
}

impl TokenKind {
    pub fn is_same(&self, other: &TokenKind) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.column + 1)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub position: Position,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, column: usize, lexeme: &str) -> Self {
        Self {
            kind,
            position: Position::new(line, column),
            lexeme: lexeme.to_string(),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lexeme)
    }
}

