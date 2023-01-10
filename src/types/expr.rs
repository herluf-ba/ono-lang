use crate::types::Token;

/// Represents a language construct that can be evaluated to a value
#[derive(Debug, Clone)]
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

/// Implement an `ExprVisitor` to recursively walk an expression tree.
/// Can be used to implement an interpreter but also things like a resolver or type checker.
pub trait ExprVisitor<T> {
    fn visit_expression(&mut self, e: &Expr) -> T;
}
