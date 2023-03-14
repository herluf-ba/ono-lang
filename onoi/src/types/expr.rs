use crate::types::Token;

use super::Stmt;

/// Represents a language construct that can be evaluated to a value
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal {
        value: Token,
    },
    Tuple {
        inners: Vec<Expr>,
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
    Variable {
        name: Token,
    },
    Assign {
        name: Token,
        expr: Box<Expr>
    },
    Block {
        statements: Vec<Stmt>,
        finally: Option<Box<Expr>>
    }
}

