use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    LEFTPAREN,
    RIGHTPAREN,
    MINUS,
    PLUS,
    SLASH,
    STAR,
    BANG,
    BANGEQUAL,
    EQUAL,
    EQUALEQUAL,
    GREATER,
    GREATEREQUAL,
    LESS,
    LESSEQUAL,
    TRUE,
    FALSE,
    AND,
    OR,
    STRING(String),
    NUMBER(f64),

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
        Self { line, column }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub position: Position,
}

impl Token {
    pub fn new(kind: TokenKind, position: Position, lexeme: &str) -> Self {
        Self {
            kind,
            position,
            lexeme: lexeme.to_string(),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lexeme)
    }
}