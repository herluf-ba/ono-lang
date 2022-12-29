use std::fmt;

use crate::lexer::Token;

#[derive(Debug)]
pub enum Expr {
    Literal {
        value: Token,
    },
    Variable {
        name: Token,
    },
    Assign {
        name: Token,
        expr: Box<Expr>,
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
    fn visit_expression(&mut self, e: &Expr) -> T;
}

#[derive(Debug)]
pub enum Stmt {
    Expression {
        expr: Expr,
    },
    Print {
        expr: Expr,
    },
    Let {
        name: Token,
        initializer: Expr,
    },
    Block {
        statements: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then: Box<Stmt>,
        eelse: Option<Box<Stmt>>,
    },
}

pub trait StmtVisitor<T> {
    fn visit_statement(&mut self, e: &Stmt) -> T;
}

pub struct ExprPrinter;
impl ExprVisitor<String> for ExprPrinter {
    fn visit_expression(&mut self, e: &Expr) -> String {
        match e {
            Expr::Literal { value } => format!("{}", value),
            Expr::Variable { name } => format!("{}", name.lexeme),
            Expr::Assign { name, expr } => {
                format!("{} = {}", name.lexeme, self.visit_expression(expr.as_ref()))
            }
            Expr::Unary { operator, expr } => {
                format!("({} {})", operator, self.visit_expression(expr.as_ref()))
            }
            Expr::Binary {
                operator,
                left,
                right,
            } => format!(
                "({} {} {})",
                operator,
                self.visit_expression(left.as_ref()),
                self.visit_expression(right.as_ref()),
            ),
            Expr::Group { expr } => format!("(group {})", self.visit_expression(expr.as_ref())),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", ExprPrinter.visit_expression(self))
    }
}
