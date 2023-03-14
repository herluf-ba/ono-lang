use crate::{
    environment::Environment,
    error::{language_error, Error, TypeError},
    types::{Expr, Stmt, TokenKind, Type},
};

pub struct Typechecker {
    scope: Environment<Type>,
}

impl Typechecker {
    pub fn new() -> Self {
        Self {
            scope: Environment::new(),
        }
    }

    pub fn check(&mut self, statements: &Vec<Stmt>) -> Result<(), Vec<Error>> {
        let mut errors = Vec::new();
        for stmt in statements {
            match self.visit_statement(&stmt) {
                Ok(_) => {}
                Err(err) => {
                    errors.push(err);
                }
            }
        }

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(())
        }
    }

    fn visit_statement(&mut self, statement: &Stmt) -> Result<(), Error> {
        match statement {
            Stmt::Expression { expr } => {
                self.visit_expression(expr)?;
            }
            Stmt::Let {
                name,
                ttype,
                initializer,
            } => {
                let initializer_type = self.visit_expression(initializer)?;
                if let Some(ttype) = ttype {
                    if initializer_type != *ttype {
                        return Err(Error::type_error(
                            TypeError::T003 {
                                declared_as: ttype.clone(),
                                initialized_as: initializer_type.clone(),
                            },
                            name.clone(),
                        ));
                    }
                }

                self.scope.define(&name.lexeme, initializer_type);
            }
        }
        Ok(())
    }

    pub fn visit_expression(&mut self, e: &Expr) -> Result<Type, Error> {
        match e {
            Expr::Literal { value } => Ok(Type::from(value)),
            Expr::Group { expr } => self.visit_expression(expr),
            Expr::Tuple { inners } => Ok(Type::Tuple(
                inners
                    .iter()
                    .map(|expr| self.visit_expression(expr))
                    .collect::<Result<Vec<Type>, Error>>()?,
            )),
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
                        Type::Bool => Ok(Type::Bool),
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
                    TokenKind::MINUS | TokenKind::STAR | TokenKind::SLASH => {
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
                        if left != right {
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
            Expr::Variable { name } => {
                if let Some(ttype) = self.scope.get(&name.lexeme) {
                    Ok(ttype.clone())
                } else {
                    Err(Error::type_error(TypeError::T004, name.clone()))
                }
            }
            Expr::Assign { name, expr } => {
                let assigned_to = self.visit_expression(expr)?;
                if let Some(declared_as) = self.scope.get(&name.lexeme) {
                    return if *declared_as != assigned_to {
                        Err(Error::type_error(
                            TypeError::T005 {
                                declared_as: declared_as.clone(),
                                assigned_to,
                            },
                            name.clone(),
                        ))
                    } else {
                        Ok(assigned_to)
                    };
                } else {
                    Err(Error::type_error(TypeError::T004, name.clone()))
                }
            }
            Expr::Block {
                statements,
                finally,
            } => {
                self.scope = self.scope.new_nested();
                for stmt in statements {
                    self.visit_statement(stmt)?;
                }
                let val = if let Some(expr) = finally {
                    self.visit_expression(expr)?
                } else {
                    Type::Tuple(Vec::new())
                };
                self.scope.pop();
                Ok(val)
            },
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
    fn tuple() -> Result<(), Error> {
        let expr = Tuple {
            inners: vec![
                Binary {
                    left: Box::new(Literal {
                        value: Token::new(NUMBER(1.0), 0, 0, "1"),
                    }),
                    operator: Token::new(PLUS, 0, 2, "+"),
                    right: Box::new(Literal {
                        value: Token::new(NUMBER(2.0), 0, 4, "2"),
                    }),
                },
                Binary {
                    left: Box::new(Literal {
                        value: Token::new(NUMBER(3.0), 0, 6, "3"),
                    }),
                    operator: Token::new(PLUS, 0, 7, "+"),
                    right: Box::new(Literal {
                        value: Token::new(NUMBER(4.0), 0, 8, "4"),
                    }),
                },
            ],
        };

        assert_eq!(
            Typechecker::new().visit_expression(&expr)?,
            Type::Tuple(vec![Type::Number, Type::Number])
        );

        Ok(())
    }

    #[test]
    fn logical() -> Result<(), Error> {
        let expr_ok = Logical {
            left: Box::new(Literal {
                value: Token::new(TRUE, 0, 0, "true"),
            }),
            operator: Token::new(OR, 0, 4, "or"),
            right: Box::new(Literal {
                value: Token::new(FALSE, 0, 6, "false"),
            }),
        };

        assert_eq!(Typechecker::new().visit_expression(&expr_ok)?, Type::Bool);

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
            Typechecker::new().visit_expression(&expr_bad),
            Err(Error::type_error(
                TypeError::T001 {
                    left: Type::Bool,
                    right: Type::Number
                },
                Token::new(OR, 0, 4, "or")
            ))
        );
        Ok(())
    }

    #[test]
    fn unary_negation() -> Result<(), Error> {
        let expr_minus_ok = Unary {
            operator: Token::new(MINUS, 0, 0, "-"),
            expr: Box::new(Literal {
                value: Token::new(NUMBER(1.0), 0, 1, "1"),
            }),
        };
        assert_eq!(
            Typechecker::new().visit_expression(&expr_minus_ok)?,
            Type::Number
        );

        let expr_minus_bad = Unary {
            operator: Token::new(MINUS, 0, 0, "-"),
            expr: Box::new(Literal {
                value: Token::new(TRUE, 0, 1, "true"),
            }),
        };
        assert_eq!(
            Typechecker::new().visit_expression(&expr_minus_bad),
            Err(Error::type_error(
                TypeError::T002 {
                    operand: Type::Bool
                },
                Token::new(MINUS, 0, 0, "-")
            ))
        );

        let expr_bang_ok = Unary {
            operator: Token::new(BANG, 0, 0, "!"),
            expr: Box::new(Literal {
                value: Token::new(FALSE, 0, 1, "false"),
            }),
        };
        assert_eq!(
            Typechecker::new().visit_expression(&expr_bang_ok)?,
            Type::Bool
        );

        let expr_bang_bad = Unary {
            operator: Token::new(BANG, 0, 0, "!"),
            expr: Box::new(Literal {
                value: Token::new(NUMBER(1.0), 0, 1, "1"),
            }),
        };
        assert_eq!(
            Typechecker::new().visit_expression(&expr_bang_bad),
            Err(Error::type_error(
                TypeError::T002 {
                    operand: Type::Number
                },
                Token::new(BANG, 0, 0, "!")
            ))
        );
        Ok(())
    }

    #[test]
    fn binary_addition() -> Result<(), Error> {
        let expr_ok = Binary {
            left: Box::new(Literal {
                value: Token::new(NUMBER(1.0), 0, 0, "1"),
            }),
            operator: Token::new(PLUS, 0, 1, "+"),
            right: Box::new(Literal {
                value: Token::new(NUMBER(2.0), 0, 2, "2"),
            }),
        };

        assert_eq!(Typechecker::new().visit_expression(&expr_ok)?, Type::Number);

        let expr_ok_string = Binary {
            left: Box::new(Literal {
                value: Token::new(STRING("foo".to_string()), 0, 0, "foo"),
            }),
            operator: Token::new(PLUS, 0, 4, "+"),
            right: Box::new(Literal {
                value: Token::new(STRING("bar".to_string()), 0, 6, "bar"),
            }),
        };

        assert_eq!(
            Typechecker::new().visit_expression(&expr_ok_string)?,
            Type::Text
        );

        let expr_bad = Binary {
            left: Box::new(Literal {
                value: Token::new(NUMBER(1.23), 0, 0, "1.23"),
            }),
            operator: Token::new(PLUS, 0, 4, "+"),
            right: Box::new(Literal {
                value: Token::new(STRING("bar".to_string()), 0, 6, "bar"),
            }),
        };
        assert_eq!(
            Typechecker::new().visit_expression(&expr_bad),
            Err(Error::type_error(
                TypeError::T001 {
                    left: Type::Number,
                    right: Type::Text
                },
                Token::new(PLUS, 0, 4, "+")
            ))
        );
        Ok(())
    }

    #[test]
    fn number_comparison() -> Result<(), Error> {
        let expr_ok = Binary {
            left: Box::new(Literal {
                value: Token::new(NUMBER(1.0), 0, 0, "1"),
            }),
            operator: Token::new(LESS, 0, 2, "<"),
            right: Box::new(Literal {
                value: Token::new(NUMBER(2.0), 0, 4, "2"),
            }),
        };

        assert_eq!(Typechecker::new().visit_expression(&expr_ok)?, Type::Bool);

        let expr_bad = Binary {
            left: Box::new(Literal {
                value: Token::new(NUMBER(1.0), 0, 0, "1"),
            }),
            operator: Token::new(LESS, 0, 2, "<"),
            right: Box::new(Literal {
                value: Token::new(STRING("foo".to_string()), 0, 4, "foo"),
            }),
        };

        assert_eq!(
            Typechecker::new().visit_expression(&expr_bad),
            Err(Error::type_error(
                TypeError::T001 {
                    left: Type::Number,
                    right: Type::Text
                },
                Token::new(LESS, 0, 2, "<")
            ))
        );
        Ok(())
    }

    #[test]
    fn equality() -> Result<(), Error> {
        let expr_ok = Binary {
            left: Box::new(Literal {
                value: Token::new(NUMBER(1.0), 0, 0, "1"),
            }),
            operator: Token::new(LESS, 0, 2, "=="),
            right: Box::new(Literal {
                value: Token::new(NUMBER(2.0), 0, 5, "2"),
            }),
        };

        assert_eq!(Typechecker::new().visit_expression(&expr_ok)?, Type::Bool);

        let expr_bad = Binary {
            left: Box::new(Literal {
                value: Token::new(NUMBER(1.0), 0, 0, "1"),
            }),
            operator: Token::new(EQUALEQUAL, 0, 2, "=="),
            right: Box::new(Literal {
                value: Token::new(STRING("foo".to_string()), 0, 5, "foo"),
            }),
        };

        assert_eq!(
            Typechecker::new().visit_expression(&expr_bad),
            Err(Error::type_error(
                TypeError::T001 {
                    left: Type::Number,
                    right: Type::Text
                },
                Token::new(EQUALEQUAL, 0, 2, "==")
            ))
        );
        Ok(())
    }
}
