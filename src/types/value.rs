use super::{Token, TokenKind};
use std::fmt::{Debug, Display};

/// The types ono supports
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Text,
    Number,
    Bool,
    Tuple(Vec<Type>),
}

impl From<&Token> for Type {
    fn from(token: &Token) -> Self {
        match &token.kind {
            TokenKind::NUMBER(_) => Type::Number,
            TokenKind::STRING(_) => Type::Text,
            TokenKind::FALSE | TokenKind::TRUE => Type::Bool,
            _ => Type::Tuple(vec![]),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Bool => write!(f, "bool"),
            Type::Text => write!(f, "text"),
            Type::Number => write!(f, "number"),
            Type::Tuple(inners) => write!(
                f,
                "({})",
                inners
                    .iter()
                    .map(|inner| format!("{}", inner))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

/// Representation of a value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Text(String),
    Number(f64),
    Tuple(Vec<Value>)
}

impl Value {
    /// null and `false` are falsy in ono. Everything else is thruthy.
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Tuple(inner) => inner.len() == 0,
            Value::Bool(val) => *val,
            _ => true,
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
            _ => Value::Tuple(vec![]),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(v) => write!(f, "{}", v),
            Value::Text(v) => write!(f, "{}", v),
            Value::Number(v) => write!(f, "{}", v),
            Value::Tuple(inners) => write!(
                f,
                "({})",
                inners
                    .iter()
                    .map(|inner| format!("{}", inner))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}
