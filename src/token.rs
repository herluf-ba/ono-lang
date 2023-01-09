use std::fmt::Display;

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
    COLON,
    SEMICOLON,
    SLASH,
    STAR,
    QUESTIONMARK,

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
    TRUE,
    FALSE,
    SOME,
    NONE,

    // Boolean operators
    AND,
    OR,

    // Keywords
    LET,
    IF,
    ELSE,
    MATCH,
    HAS,
    TRAIT,
    ENUM,
    OBJ,
    FUN,
    RETURN,
    SELF,
    WHILE,
    FOR,
    IN,

    // To be removed
    PRINT,
    NULL,

    // Internal
    EOF,
    UNKNOWN,
}

impl TokenKind {
    pub fn is_same(&self, other: &TokenKind) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
