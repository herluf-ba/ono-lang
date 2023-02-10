use crate::types::Token;

/// Represents a language construct that can be evaluated to a value
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal {
        value: Token,
    },
    Unary {
        operator: Token,
        expr: Box<Expr>,
    },
    Binary {
        operator: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Logical {
        operator: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Group {
        expr: Box<Expr>,
    },
}

