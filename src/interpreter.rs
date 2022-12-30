use crate::{
    ast::*,
    environment::Environment,
    error::{Error, RuntimeError, SyntaxError, TypeError},
    lexer::{Token, TokenKind},
};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Value {
    Range { to: f64, from: f64, step_by: f64 },
    Bool(bool),
    Text(String),
    Number(f64),
    Null,
}

impl Value {
    fn is_equal(&self, other: &Value) -> bool {
        match self {
            Value::Range { to, from, step_by } => match other {
                Value::Range {
                    to: o_to,
                    from: o_from,
                    step_by: o_step_by,
                } => to == o_to && from == o_from && o_step_by == step_by,
                _ => false,
            },
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
            Value::Range {
                to: _,
                from: _,
                step_by: _,
            } => "range",
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
            Value::Range { to, from, step_by } => write!(
                f,
                "{}..{}{}",
                from,
                if *step_by != 1.0 {
                    format!("{}..", step_by)
                } else {
                    "".to_string()
                },
                to
            ),
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
        self.environment.nest();

        for statement in statements {
            self.visit_statement(&statement)?;
        }

        self.environment.pop();
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
            Expr::Range {
                token,
                from,
                step_by,
                to,
            } => {
                let from = self.visit_expression(from)?;
                let to = self.visit_expression(to)?;

                let step_by = if let Some(step_by) = step_by {
                    Some(self.visit_expression(&step_by)?)
                } else {
                    None
                };

                let from_num = match from {
                    Value::Number(num) => num,
                    _ => {
                        return Err(Error::type_error(
                            TypeError::T003 { from, to, step_by },
                            token.clone(),
                        ))
                    }
                };

                let to_num = match to {
                    Value::Number(num) => num,
                    _ => {
                        return Err(Error::type_error(
                            TypeError::T003 { from, to, step_by },
                            token.clone(),
                        ))
                    }
                };

                let step_by_num = match step_by {
                    None => 1.0,
                    Some(value) => match value {
                        Value::Number(num) => num,
                        _ => {
                            return Err(Error::type_error(
                                TypeError::T003 {
                                    from,
                                    to,
                                    step_by: Some(value),
                                },
                                token.clone(),
                            ))
                        }
                    },
                };

                Ok(Value::Range {
                    to: to_num,
                    from: from_num,
                    step_by: step_by_num,
                })
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
            Stmt::While { condition, body } => {
                while self.visit_expression(condition)?.is_truthy() {
                    self.visit_statement(body)?;
                }
            }
            Stmt::For {
                identifier,
                range,
                body,
            } => {
                if let Value::Range { to, from, step_by } = self.visit_expression(range)? {
                    if (step_by > 0.0 && to <= from) || (step_by < 0.0 && from <= to) {
                        return Err(Error::runtime_error(
                            RuntimeError::R003 { from, to, step_by },
                            match range {
                                Expr::Range { token, from: _, step_by: _, to:_ } => token.clone(),
                                _ => panic!("Stmt::For created with faulty range expression. This is an internal ono error")
                            }
                        ));
                    }

                    let mut num = to.min(from);
                    let dest = to.max(from);
                    self.environment
                        .define(&identifier.lexeme, Value::Number(num));
                    while num < dest {
                        self.environment
                            .assign(&identifier.lexeme, Value::Number(num))
                            .unwrap();
                        self.visit_statement(&body)?;
                        num += step_by;
                    }
                }
            }
        };
        Ok(())
    }
}
