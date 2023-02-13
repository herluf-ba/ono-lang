use crate::{
    error::{language_error, Error, TypeError},
    types::{Expr, TokenKind, Type},
};

pub struct Typechecker;

impl Typechecker {
    pub fn new() -> Self {
        Self {}
    }

    pub fn check(&mut self, expression: Expr) -> Result<Type, Vec<Error>> {
        match self.visit_expression(&expression) {
            Ok(t) => Ok(t),
            Err(err) => Err(vec![err]),
        }
    }

    fn visit_expression(&mut self, e: &Expr) -> Result<Type, Error> {
        match e {
            Expr::Literal { value } => Ok(Type::from(value)),
            Expr::Group { expr } => self.visit_expression(expr),
            Expr::Logical {
                operator,
                left,
                right,
            } => match (self.visit_expression(left)?, self.visit_expression(right)?) {
                (Type::Bool, Type::Bool) => Ok(Type::Bool),
                (left, right) => Err(Error::type_error(
                    TypeError::T001 { left, right },
                    operator.clone(),
                )),
            },
            Expr::Unary { operator, expr } => {
                let operand = self.visit_expression(expr)?;
                match operator.kind {
                    TokenKind::BANG => match operand {
                        Type::Null | Type::Bool => Ok(Type::Bool),
                        _ => Err(Error::type_error(
                            TypeError::T002 { operand },
                            operator.clone(),
                        )),
                    },
                    TokenKind::MINUS => match operand {
                        Type::Number => Ok(Type::Number),
                        _ => Err(Error::type_error(
                            TypeError::T002 { operand },
                            operator.clone(),
                        )),
                    },
                    _ => language_error(&format!("unknown unary operator")),
                }
            }
            Expr::Binary {
                operator,
                left,
                right,
            } => {
                let left = self.visit_expression(left)?;
                let right = self.visit_expression(right)?;
                match operator.kind {
                    TokenKind::PLUS => match (left, right) {
                        (Type::Number, Type::Number) => Ok(Type::Number),
                        (Type::Text, Type::Text) => Ok(Type::Text),
                        (left, right) => Err(Error::type_error(
                            TypeError::T001 { left, right },
                            operator.clone(),
                        )),
                    },
                    TokenKind::MINUS
                    | TokenKind::STAR
                    | TokenKind::SLASH => {
                        if left != Type::Number || right != Type::Number {
                            Err(Error::type_error(
                                TypeError::T001 { left, right },
                                operator.clone(),
                            ))
                        } else {
                            Ok(Type::Number)
                        }
                    }
                    TokenKind::LESS
                    | TokenKind::LESSEQUAL
                    | TokenKind::GREATER
                    | TokenKind::GREATEREQUAL => {
                        if left != Type::Number || right != Type::Number {
                            Err(Error::type_error(
                                TypeError::T001 { left, right },
                                operator.clone(),
                            ))
                        } else {
                            Ok(Type::Bool)
                        }
                    }
                    TokenKind::EQUALEQUAL | TokenKind::BANGEQUAL => {
                        if left != Type::Bool || right != Type::Bool {
                            Err(Error::type_error(
                                TypeError::T001 { left, right },
                                operator.clone(),
                            ))
                        } else {
                            Ok(Type::Bool)
                        }
                    }
                    _ => language_error(&format!("unknown binary operator '{}'", operator.lexeme)),
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{Expr, Token, TokenKind, Type};
    use pretty_assertions::assert_eq;
    use Expr::*;
    use TokenKind::*;

    #[test]
    fn logical() -> Result<(), Vec<Error>> {
        let expr_ok = Logical {
            left: Box::new(Literal {
                value: Token::new(TRUE, 0, 0, "true"),
            }),
            operator: Token::new(OR, 0, 4, "or"),
            right: Box::new(Literal {
                value: Token::new(FALSE, 0, 6, "false"),
            }),
        };

        assert_eq!(Typechecker::new().check(expr_ok)?, Type::Bool);

        let expr_bad = Logical {
            left: Box::new(Literal {
                value: Token::new(TRUE, 0, 0, "true"),
            }),
            operator: Token::new(OR, 0, 4, "or"),
            right: Box::new(Literal {
                value: Token::new(NUMBER(1.0), 0, 6, "1"),
            }),
        };

        assert_eq!(
            Typechecker::new().check(expr_bad),
            Err(vec![Error::type_error(
                TypeError::T001 {
                    left: Type::Bool,
                    right: Type::Number
                },
                Token::new(OR, 0, 4, "or")
            )])
        );
        Ok(())
    }

    #[test]
    fn unary_negation() -> Result<(), Vec<Error>> {
        let expr_minus_ok = Unary {
            operator: Token::new(MINUS, 0, 0, "-"),
            expr: Box::new(Literal {
                value: Token::new(NUMBER(1.0), 0, 1, "1"),
            }),
        };
        assert_eq!(Typechecker::new().check(expr_minus_ok)?, Type::Number);

        let expr_minus_bad = Unary {
            operator: Token::new(MINUS, 0, 0, "-"),
            expr: Box::new(Literal {
                value: Token::new(TRUE, 0, 1, "true"),
            }),
        };
        assert_eq!(
            Typechecker::new().check(expr_minus_bad),
            Err(vec![Error::type_error(
                TypeError::T002 {
                    operand: Type::Bool
                },
                Token::new(MINUS, 0, 0, "-")
            )])
        );

        let expr_bang_ok = Unary {
            operator: Token::new(BANG, 0, 0, "!"),
            expr: Box::new(Literal {
                value: Token::new(FALSE, 0, 1, "false"),
            }),
        };
        assert_eq!(Typechecker::new().check(expr_bang_ok)?, Type::Bool);

        let expr_bang_bad = Unary {
            operator: Token::new(BANG, 0, 0, "!"),
            expr: Box::new(Literal {
                value: Token::new(NUMBER(1.0), 0, 1, "1"),
            }),
        };
        assert_eq!(
            Typechecker::new().check(expr_bang_bad),
            Err(vec![Error::type_error(
                TypeError::T002 {
                    operand: Type::Number
                },
                Token::new(BANG, 0, 0, "!")
            )])
        );
        Ok(())
    }
}
