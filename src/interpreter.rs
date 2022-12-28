use crate::{
    ast::*,
    environment::Environment,
    error::{Error, ErrorKind},
    lexer::{Token, TokenKind},
};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Text(String),
    Number(f64),
    Null,
}

impl Value {
    fn is_equal(&self, other: &Value) -> bool {
        match self {
            Value::Bool(s) => match other {
                Value::Bool(o) => s == o,
                _ => false,
            },
            Value::Text(s) => match other {
                Value::Text(o) => s == o,
                _ => false,
            },
            Value::Number(s) => match other {
                Value::Number(o) => s == o,
                _ => false,
            },
            Value::Null => match other {
                Value::Null => true,
                _ => false,
            },
        }
    }

    fn display_type(&self) -> String {
        String::from(match self {
            Value::Bool(_) => "boolean",
            Value::Text(_) => "string",
            Value::Number(_) => "number",
            Value::Null => "null",
        })
    }
}

impl From<&Token> for Value {
    fn from(token: &Token) -> Self {
        match &token.kind {
            TokenKind::NUMBER(num) => Value::Number(*num),
            TokenKind::STRING(s) => Value::Text(s.clone()),
            TokenKind::FALSE => Value::Bool(false),
            TokenKind::TRUE => Value::Bool(true),
            _ => Value::Null,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(v) => write!(f, "{}", v),
            Value::Text(v) => write!(f, "{}", v),
            Value::Number(v) => write!(f, "{}", v),
            Value::Null => write!(f, "null"),
        }
    }
}

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), Error> {
        for statement in statements {
            self.visit_statement(&statement)?;
        }
        Ok(())
    }

    fn unary(operator: &Token, value: Value) -> Result<Value, Error> {
        match operator.kind {
            TokenKind::BANG => match value {
                Value::Null => Ok(Value::Bool(false)),
                Value::Bool(v) => Ok(Value::Bool(!v)),
                _ => Err(Error::from_token(
                    operator,
                    ErrorKind::RuntimeError,
                    &format!("Unable to negate value of type '{}'", value.display_type()),
                )),
            },
            TokenKind::MINUS => match value {
                Value::Number(v) => Ok(Value::Number(-v)),
                _ => Err(Error::from_token(
                    operator,
                    ErrorKind::RuntimeError,
                    &format!("Unable to negate value of type '{}'", value.display_type()),
                )),
            },
            _ => Err(Error::from_token(
                operator,
                ErrorKind::RuntimeError,
                "Unknown unary operator encountered",
            )),
        }
    }

    fn binary(operator: &Token, left: Value, right: Value) -> Result<Value, Error> {
        let mismatch_error = Err(Error::from_token(
            operator,
            ErrorKind::RuntimeError,
            &format!(
                "Cannot perform '{}' on a '{}' and a '{}'",
                operator.lexeme,
                left.display_type(),
                right.display_type()
            ),
        ));

        match operator.kind {
            TokenKind::PLUS => match left {
                Value::Number(l) => match right {
                    Value::Number(r) => Ok(Value::Number(l + r)),
                    _ => mismatch_error,
                },
                Value::Text(l) => match right {
                    Value::Text(r) => Ok(Value::Text(format!("{}{}", l, r))),
                    _ => mismatch_error,
                },
                _ => mismatch_error,
            },
            TokenKind::MINUS => match left {
                Value::Number(l) => match right {
                    Value::Number(r) => Ok(Value::Number(l - r)),
                    _ => mismatch_error,
                },
                _ => mismatch_error,
            },
            TokenKind::STAR => match left {
                Value::Number(l) => match right {
                    Value::Number(r) => Ok(Value::Number(l * r)),
                    _ => mismatch_error,
                },
                _ => mismatch_error,
            },
            TokenKind::SLASH => match left {
                Value::Number(l) => match right {
                    Value::Number(r) => {
                        if r != 0.0 {
                            Ok(Value::Number(l / r))
                        } else {
                            Err(Error::from_token(
                                operator,
                                ErrorKind::RuntimeError,
                                "Division by zero",
                            ))
                        }
                    }
                    _ => mismatch_error,
                },
                _ => mismatch_error,
            },
            TokenKind::GREATER => match left {
                Value::Number(l) => match right {
                    Value::Number(r) => Ok(Value::Bool(l > r)),
                    _ => mismatch_error,
                },
                _ => mismatch_error,
            },
            TokenKind::GREATEREQUAL => match left {
                Value::Number(l) => match right {
                    Value::Number(r) => Ok(Value::Bool(l >= r)),
                    _ => mismatch_error,
                },
                _ => mismatch_error,
            },
            TokenKind::EQUALEQUAL => Ok(Value::Bool(left.is_equal(&right))),
            TokenKind::BANGEQUAL => Ok(Value::Bool(!left.is_equal(&right))),
            TokenKind::LESS => match left {
                Value::Number(l) => match right {
                    Value::Number(r) => Ok(Value::Bool(l < r)),
                    _ => mismatch_error,
                },
                _ => mismatch_error,
            },
            TokenKind::LESSEQUAL => match left {
                Value::Number(l) => match right {
                    Value::Number(r) => Ok(Value::Bool(l <= r)),
                    _ => mismatch_error,
                },
                _ => mismatch_error,
            },
            _ => Err(Error::from_token(
                operator,
                ErrorKind::SyntaxError,
                "Unknown operator",
            )),
        }
    }
}

impl ExprVisitor<Result<Value, Error>> for Interpreter {
    fn visit_expression(&mut self, e: &Expr) -> Result<Value, Error> {
        match e {
            Expr::Literal { value } => Ok(Value::from(value)),
            Expr::Variable { name } => match self.environment.get(&name.lexeme) {
                Some(value) => Ok((*value).clone()),
                None => Err(Error::from_token(
                    name,
                    ErrorKind::RuntimeError,
                    &format!("'{}' is undefined here", name.lexeme),
                )),
            },
            Expr::Assign { name, expr } => {
                let val = self.visit_expression(expr)?;
                match self.environment.assign(&name.lexeme, val.clone()) {
                    Ok(_) => {}
                    Err(_) => {
                        return Err(Error::from_token(
                            name,
                            ErrorKind::RuntimeError,
                            &format!("unknown identifier '{}' mentioned here", name.lexeme),
                        ))
                    }
                }

                Ok(val)
            }
            Expr::Unary { operator, expr } => {
                let val = self.visit_expression(expr)?;
                Interpreter::unary(operator, val)
            }
            Expr::Binary {
                operator,
                left,
                right,
            } => {
                let left = self.visit_expression(left)?;
                let right = self.visit_expression(right)?;
                Interpreter::binary(operator, left, right)
            }
            Expr::Group { expr } => self.visit_expression(expr),
        }
    }
}
impl StmtVisitor<Result<(), Error>> for Interpreter {
    fn visit_statement(&mut self, s: &Stmt) -> Result<(), Error> {
        match s {
            Stmt::Expression { expr } => {
                self.visit_expression(expr)?;
            }
            Stmt::Print { expr } => println!("{}", self.visit_expression(expr)?),
            Stmt::Let { name, initializer } => {
                let value = self.visit_expression(initializer)?;
                self.environment.define(&name.lexeme, value)
            }
        };
        Ok(())
    }
}
