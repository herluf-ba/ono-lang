use crate::{
    error::{language_error, Error, RuntimeError},
    types::{Expr, TokenKind, Value},
};

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&mut self, expression: Expr) -> Result<Value, Error> {
        self.visit_expression(&expression)
    }

    fn visit_expression(&mut self, e: &Expr) -> Result<Value, Error> {
        match e {
            Expr::Literal { value } => Ok(Value::from(value)),
            Expr::Group { expr } => self.visit_expression(expr),
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
                        Value::Null => Ok(Value::Bool(false)),
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
                            _ => language_error(&format!(
                                "unsupported binary operator '{}'",
                                operator.lexeme
                            )),
                        }
                    }
                }

                if let Value::Text(ref l) = left {
                    if let Value::Text(r) = right {
                        match operator.kind {
                            TokenKind::PLUS => return Ok(Value::Text(format!("{}{}", l, r))),
                            _ => language_error(&format!(
                                "unsupported binary operator '{}'",
                                operator.lexeme
                            )),
                        }
                    }
                }

                match operator.kind {
                    TokenKind::EQUALEQUAL => return Ok(Value::Bool(left == right)),
                    TokenKind::BANGEQUAL => return Ok(Value::Bool(left != right)),
                    _ => language_error(&format!("unknown binary operator '{}'", operator.lexeme)),
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use crate::types::{Expr, Token, TokenKind, Value};
    use Expr::*;
    use TokenKind::*;

    #[test]
    fn or() -> Result<(), Error> {
        let expr = Logical {
            left: Box::new(Literal { value: Token::new(TRUE, 0, 0, "true"), }),
            operator: Token::new(OR, 0, 4, "or"),
            right: Box::new(Literal { value: Token::new(FALSE, 0, 6, "false"), }),
        };

        let result = Interpreter::new().interpret(expr)?;
        assert_eq!(result, Value::Bool(true));
        Ok(())
    }
    
    #[test]
    fn addition() -> Result<(), Error> {
        let expr = Binary {
            left: Box::new(Literal { value: Token::new(NUMBER(1.0), 0, 0, "1"), }),
            operator: Token::new(PLUS, 0, 2, "+"),
            right: Box::new(Literal { value: Token::new(NUMBER(2.0), 0, 4, "2"), }),
        };

        let result = Interpreter::new().interpret(expr)?;
        assert_eq!(result, Value::Number(3.0));
        Ok(())
    }
   
    #[test]
    fn string_addition() -> Result<(), Error> {
        let expr = Binary {
            left: Box::new(Literal { value: Token::new(STRING("foo".to_string()), 0, 0, "foo"), }),
            operator: Token::new(PLUS, 0, 4, "+"),
            right: Box::new(Literal { value: Token::new(STRING("bar".to_string()), 0, 5, "bar"), }),
        };

        let result = Interpreter::new().interpret(expr)?;
        assert_eq!(result, Value::Text("foobar".to_string()));
        Ok(())
    }
    
    #[test]
    fn less() -> Result<(), Error> {
        let expr = Binary {
            left: Box::new(Literal { value: Token::new(NUMBER(1.0), 0, 0, "1"), }),
            operator: Token::new(LESS, 0, 2, "<"),
            right: Box::new(Literal { value: Token::new(NUMBER(2.0), 0, 4, "2"), }),
        };

        let result = Interpreter::new().interpret(expr)?;
        assert_eq!(result, Value::Bool(true));
        Ok(())
    }

    #[test]
    fn errors_on_division_by_zero() -> Result<(), Error> {
        let expr = Binary {
            left: Box::new(Literal { value: Token::new(NUMBER(1.0), 0, 0, "1"), }),
            operator: Token::new(SLASH, 0, 2, "/"),
            right: Box::new(Literal { value: Token::new(NUMBER(0.0), 0, 4, "0"), }),
        };

        let result = Interpreter::new().interpret(expr);
        assert_eq!(result, Err(Error::runtime_error(RuntimeError::R001, Token::new(SLASH, 0, 2, "/"))));
        Ok(())
    }
}
