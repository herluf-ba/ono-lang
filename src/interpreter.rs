use crate::{
    ast::*,
    environment::Environment,
    error::{Error, RuntimeError, SyntaxError, TypeError},
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

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(val) => *val,
            _ => true,
        }
    }

    pub fn display_type(&self) -> String {
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

    pub fn execute_block(&mut self, statements: &Vec<Stmt>) -> Result<(), Error> {
        let enclosing_env = self.environment.clone();
        self.environment = Environment::new_nested(&enclosing_env);

        for statement in statements {
            self.visit_statement(&statement)?;
        }

        self.environment = enclosing_env;
        Ok(())
    }

    fn unary(operator: &Token, value: Value) -> Result<Value, Error> {
        match operator.kind {
            TokenKind::BANG => match value {
                Value::Null => Ok(Value::Bool(false)),
                Value::Bool(v) => Ok(Value::Bool(!v)),
                _ => Err(Error::type_error(
                    TypeError::T001 { operand: value },
                    operator.clone(),
                )),
            },
            TokenKind::MINUS => match value {
                Value::Number(v) => Ok(Value::Number(-v)),
                _ => Err(Error::type_error(
                    TypeError::T001 { operand: value },
                    operator.clone(),
                )),
            },
            _ => Err(Error::syntax_error(SyntaxError::S001, operator.clone())),
        }
    }

    fn binary(operator: &Token, left: Value, right: Value) -> Result<Value, Error> {
        let mismatch_error = Err(Error::type_error(
            TypeError::T002 {
                left: left.clone(),
                right: right.clone(),
            },
            operator.clone(),
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
                            Err(Error::runtime_error(RuntimeError::R002, operator.clone()))
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
            _ => Err(Error::syntax_error(SyntaxError::S001, operator.clone())),
        }
    }
}

impl ExprVisitor<Result<Value, Error>> for Interpreter {
    fn visit_expression(&mut self, e: &Expr) -> Result<Value, Error> {
        match e {
            Expr::Literal { value } => Ok(Value::from(value)),
            Expr::Variable { name } => match self.environment.get(&name.lexeme) {
                Some(value) => Ok((*value).clone()),
                None => Err(Error::runtime_error(RuntimeError::R001, name.clone())),
            },
            Expr::Assign { name, expr } => {
                let val = self.visit_expression(expr)?;
                match self.environment.assign(&name.lexeme, val.clone()) {
                    Ok(_) => {}
                    Err(_) => return Err(Error::runtime_error(RuntimeError::R001, name.clone())),
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
            Stmt::Block { statements } => {
                self.execute_block(statements)?;
            }
            Stmt::If {
                condition,
                then,
                eelse,
            } => {
                if self
                    .visit_expression(condition)?
                    .is_equal(&Value::Bool(true))
                {
                    self.visit_statement(then)?;
                } else if let Some(eelse) = eelse {
                    self.visit_statement(eelse)?;
                }
            }
        };
        Ok(())
    }
}
