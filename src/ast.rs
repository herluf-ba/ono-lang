use std::fmt;

use crate::lexer::Token;

#[derive(Debug, Clone)]
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
    Logical {
        operator: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Group {
        expr: Box<Expr>,
    },
    Range {
        token: Token,
        from: Box<Expr>,
        step_by: Option<Box<Expr>>,
        to: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
}

// Expressions implement the visitor pattern
pub trait ExprVisitor<T> {
    fn visit_expression(&mut self, e: &Expr) -> T;
}

#[derive(Debug, Clone)]
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
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    For {
        identifier: Token,
        range: Expr,
        body: Box<Stmt>,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    Return {
        keyword: Token,
        expr: Option<Expr>,
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
            Expr::Range {
                to,
                from,
                step_by,
                token: _,
            } => format!(
                "{}..{}{}",
                self.visit_expression(from.as_ref()),
                if let Some(step_by) = step_by {
                    format!("{}..", self.visit_expression(step_by.as_ref()))
                } else {
                    "".to_string()
                },
                self.visit_expression(to.as_ref())
            ),
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
            Expr::Logical {
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
            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => format!(
                "{}({})",
                self.visit_expression(callee.as_ref()),
                arguments
                    .iter()
                    .map(|e| { self.visit_expression(e) })
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", ExprPrinter.visit_expression(self))
    }
}
