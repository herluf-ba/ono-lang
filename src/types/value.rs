use std::fmt::{Debug, Display};
use super::{Token, TokenKind};

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Text(String),
    Number(f64),
    Null,
}

impl Value {
    pub fn is_equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Bool(s), Value::Bool(o)) => s == o,
            (Value::Text(s), Value::Text(o)) => s == o,
            (Value::Number(s), Value::Number(o)) => s == o,
            (Value::Null, Value::Null) => true,
            (_, _) => false,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(val) => *val,
            _ => true,
        }
    }

    pub fn display_type(&self) -> String {
        match self {
            Value::Bool(_) => "boolean".to_string(),
            Value::Text(_) => "string".to_string(),
            Value::Number(_) => "number".to_string(),
            Value::Null => "null".to_string(),
        }
    }
}

impl From<&Token> for Value {
    fn from(token: &Token) -> Self {
        match &token.kind {
            TokenKind::NUMBER(num) => Value::Number(*num),
            TokenKind::STRING(s) => Value::Text(s.clone()),
            TokenKind::FALSE => Value::Bool(false),
            TokenKind::TRUE => Value::Bool(true),
            _ => Value::Null,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(v) => write!(f, "{}", v),
            Value::Text(v) => write!(f, "{}", v),
            Value::Number(v) => write!(f, "{}", v),
            Value::Null => write!(f, "null"),
        }
    }
}
