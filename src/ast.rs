use std::fmt;

use crate::lexer::Token;

#[derive(Debug)]
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
    Group {
        expr: Box<Expr>,
    },
}

// Expressions implement the visitor pattern
pub trait ExprVisitor<T> {
    fn visit(&mut self, e: &Expr) -> T;
}

pub struct ExprPrinter;
impl ExprVisitor<String> for ExprPrinter {
    fn visit(&mut self, e: &Expr) -> String {
        match e {
            Expr::Literal { value } => format!("{}", value),
            Expr::Unary { operator, expr } => {
                format!("{} {}", operator, self.visit(expr.as_ref()))
            }
            Expr::Binary {
                operator,
                left,
                right,
            } => format!(
                "{} {} {}",
                self.visit(left.as_ref()),
                operator,
                self.visit(right.as_ref()),
            ),
            Expr::Group { expr } => format!("({})", self.visit(expr.as_ref())),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", ExprPrinter.visit(self))
    }
}
