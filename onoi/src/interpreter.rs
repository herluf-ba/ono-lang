use crate::{
    environment::Environment,
    error::{language_error, Error, RuntimeError},
    types::{Expr, Stmt, TokenKind, Value},
};

pub struct Interpreter {
    scope: Environment<Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            scope: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<(), Vec<Error>> {
        let mut errors = Vec::new();
        for stmt in statements {
            match self.visit_statement(&stmt) {
                Ok(_) => {}
                Err(error) => {
                    errors.push(error);
                }
            }
        }
        print!("{:?}", self.scope);

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(())
        }
    }

    pub fn visit_statement(&mut self, statement: &Stmt) -> Result<(), Error> {
        match statement {
            Stmt::Expression { expr } => {
                self.visit_expression(expr)?;
            }
            Stmt::Let {
                name,
                ttype: _,
                initializer,
            } => {
                let value = self.visit_expression(initializer)?;
                self.scope.define(&name.lexeme, value);
            }
        }

        Ok(())
    }

    pub fn visit_expression(&mut self, e: &Expr) -> Result<Value, Error> {
        match e {
            Expr::Literal { value } => Ok(Value::from(value)),
            Expr::Group { expr } => self.visit_expression(expr),
            Expr::Tuple { inners } => Ok(Value::Tuple(
                inners
                    .iter()
                    .map(|expr| self.visit_expression(expr))
                    .collect::<Result<Vec<Value>, Error>>()?,
            )),
            Expr::Logical {
                operator,
                left,
                right,
            } => {
                let left = self.visit_expression(left)?;
                let left_is_true = left.is_truthy();
                match operator.kind {
                    TokenKind::OR if left_is_true => Ok(left),
                    TokenKind::AND if !left_is_true => Ok(left),
                    _ => self.visit_expression(right),
                }
            }
            Expr::Unary { operator, expr } => {
                let val = self.visit_expression(expr)?;
                match operator.kind {
                    TokenKind::BANG => match val {
                        Value::Bool(v) => Ok(Value::Bool(!v)),
                        _ => language_error(&format!("non-negateable value")),
                    },
                    TokenKind::MINUS => match val {
                        Value::Number(v) => Ok(Value::Number(-v)),
                        _ => language_error(&format!("non-negateable value")),
                    },
                    _ => language_error(&format!("Unknown unary operator")),
                }
            }
            Expr::Binary {
                operator,
                left,
                right,
            } => {
                let left = self.visit_expression(left)?;
                let right = self.visit_expression(right)?;

                if let Value::Number(l) = left {
                    if let Value::Number(r) = right {
                        match operator.kind {
                            TokenKind::PLUS => return Ok(Value::Number(l + r)),
                            TokenKind::MINUS => return Ok(Value::Number(l - r)),
                            TokenKind::STAR => return Ok(Value::Number(l * r)),
                            TokenKind::SLASH => {
                                return if r == 0.0 {
                                    Err(Error::runtime_error(RuntimeError::R001, operator.clone()))
                                } else {
                                    Ok(Value::Number(l / r))
                                }
                            }
                            TokenKind::LESS => return Ok(Value::Bool(l < r)),
                            TokenKind::LESSEQUAL => return Ok(Value::Bool(l <= r)),
                            TokenKind::GREATER => return Ok(Value::Bool(l > r)),
                            TokenKind::GREATEREQUAL => return Ok(Value::Bool(l >= r)),
                            _ => {}
                        }
                    }
                }

                if let Value::Text(ref l) = left {
                    if let Value::Text(ref r) = right {
                        match operator.kind {
                            TokenKind::PLUS => return Ok(Value::Text(format!("{}{}", l, r))),
                            _ => {}
                        }
                    }
                }

                match operator.kind {
                    TokenKind::EQUALEQUAL => return Ok(Value::Bool(left == right)),
                    TokenKind::BANGEQUAL => return Ok(Value::Bool(left != right)),
                    _ => language_error(&format!("unknown binary operator '{}'", operator.lexeme)),
                }
            },
            Expr::Variable { name } => {
                if let Some(value) = self.scope.get(&name.lexeme) {
                    Ok(value.clone())
                } else {
                    language_error(
                        "undefined variable that was not type checked"
                    );
                }
            }
            Expr::Assign { name, expr } => {
                let value = self.visit_expression(expr)?;
                if let Err(_) = self.scope.assign(&name.lexeme, value.clone()) {
                    language_error(&format!("assignment target '{}' is not in scope", name.lexeme))
                }
                Ok(value)
            }, 
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{Expr, Token, TokenKind, Value};
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

        let result = Interpreter::new().visit_expression(&expr)?;
        assert_eq!(
            result,
            Value::Tuple(vec![Value::Number(3.0), Value::Number(7.0)])
        );
        Ok(())
    }

    #[test]
    fn or() -> Result<(), Error> {
        let expr = Logical {
            left: Box::new(Literal {
                value: Token::new(TRUE, 0, 0, "true"),
            }),
            operator: Token::new(OR, 0, 4, "or"),
            right: Box::new(Literal {
                value: Token::new(FALSE, 0, 6, "false"),
            }),
        };

        let result = Interpreter::new().visit_expression(&expr)?;
        assert_eq!(result, Value::Bool(true));
        Ok(())
    }

    #[test]
    fn addition() -> Result<(), Error> {
        let expr = Binary {
            left: Box::new(Literal {
                value: Token::new(NUMBER(1.0), 0, 0, "1"),
            }),
            operator: Token::new(PLUS, 0, 2, "+"),
            right: Box::new(Literal {
                value: Token::new(NUMBER(2.0), 0, 4, "2"),
            }),
        };

        let result = Interpreter::new().visit_expression(&expr)?;
        assert_eq!(result, Value::Number(3.0));
        Ok(())
    }

    #[test]
    fn string_addition() -> Result<(), Error> {
        let expr = Binary {
            left: Box::new(Literal {
                value: Token::new(STRING("foo".to_string()), 0, 0, "foo"),
            }),
            operator: Token::new(PLUS, 0, 4, "+"),
            right: Box::new(Literal {
                value: Token::new(STRING("bar".to_string()), 0, 5, "bar"),
            }),
        };

        let result = Interpreter::new().visit_expression(&expr)?;
        assert_eq!(result, Value::Text("foobar".to_string()));
        Ok(())
    }

    #[test]
    fn less() -> Result<(), Error> {
        let expr = Binary {
            left: Box::new(Literal {
                value: Token::new(NUMBER(1.0), 0, 0, "1"),
            }),
            operator: Token::new(LESS, 0, 2, "<"),
            right: Box::new(Literal {
                value: Token::new(NUMBER(2.0), 0, 4, "2"),
            }),
        };

        let result = Interpreter::new().visit_expression(&expr)?;
        assert_eq!(result, Value::Bool(true));
        Ok(())
    }

    #[test]
    fn errors_on_division_by_zero() -> Result<(), Error> {
        let expr = Binary {
            left: Box::new(Literal {
                value: Token::new(NUMBER(1.0), 0, 0, "1"),
            }),
            operator: Token::new(SLASH, 0, 2, "/"),
            right: Box::new(Literal {
                value: Token::new(NUMBER(0.0), 0, 4, "0"),
            }),
        };

        let result = Interpreter::new().visit_expression(&expr);
        assert_eq!(
            result,
            Err(Error::runtime_error(
                RuntimeError::R001,
                Token::new(SLASH, 0, 2, "/")
            ))
        );
        Ok(())
    }
}
