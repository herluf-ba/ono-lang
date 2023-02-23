use super::Expr;

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expression { expr: Expr }
}


