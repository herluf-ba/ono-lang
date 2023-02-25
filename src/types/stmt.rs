use super::{Expr, Type, Token};

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expression { expr: Expr },
    Let { name: Token, ttype: Option<Type>, initializer: Expr }
}


