use crate::{
    ast::*,
    environment::Environment,
    error::{Error, RuntimeError, SyntaxError, TypeError},
    functions::{self, Func},
    functions::{Function, NativeFunction},
    lexer::{Token, TokenKind},
};
use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    rc::Rc,
};

#[derive(Debug, Clone)]
pub enum Value {
    NativeFunction(NativeFunction),
    Function(Function),
    Range { to: f64, from: f64, step_by: f64 },
    Bool(bool),
    Text(String),
    Number(f64),
    Null,
}

impl Value {
    fn is_equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::NativeFunction(f1), Value::NativeFunction(f2)) => f1.name == f2.name,
            (Value::Function(f1), Value::Function(f2)) => f1.name.lexeme == f2.name.lexeme,
            (Value::Bool(s), Value::Bool(o)) => s == o,
            (Value::Text(s), Value::Text(o)) => s == o,
            (Value::Number(s), Value::Number(o)) => s == o,
            (Value::Null, Value::Null) => true,
            (
                Value::Range { to, from, step_by },
                Value::Range {
                    to: o_to,
                    from: o_from,
                    step_by: o_step_by,
                },
            ) => to == o_to && from == o_from && o_step_by == step_by,
            (_, _) => false,
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
        match self {
            Value::Function(func) => format!("function<{}>", func.name),
            Value::NativeFunction(func) => format!("function<{}>", func.name),
            Value::Bool(_) => "boolean".to_string(),
            Value::Range {
                to: _,
                from: _,
                step_by: _,
            } => "range".to_string(),
            Value::Text(_) => "string".to_string(),
            Value::Number(_) => "number".to_string(),
            Value::Null => "null".to_string(),
        }
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
            Value::Function(func) => write!(f, "{}({})", func.name, func.params.join(", ")),
            Value::NativeFunction(func) => write!(f, "NativeFunction<{}>", func.name),
        }
    }
}

pub struct Interpreter {
    pub environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(functions::create_global_environment())),
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

    fn prepare_func_args(
        &mut self,
        expected: usize,
        arguments: &Vec<Expr>,
        open_paren: &Token,
    ) -> Result<Vec<Value>, Error> {
        if arguments.len() != expected {
            return Err(Error::type_error(
                TypeError::T005 {
                    expected,
                    received: arguments.len(),
                },
                open_paren.clone(),
            ));
        }

        Ok(arguments
            .iter()
            .map(|arg| self.visit_expression(arg))
            .collect::<Result<Vec<Value>, Error>>()?)
    }
}

impl ExprVisitor<Result<Value, Error>> for Interpreter {
    fn visit_expression(&mut self, e: &Expr) -> Result<Value, Error> {
        match e {
            Expr::Literal { value } => Ok(Value::from(value)),
            Expr::Variable { name } => match self.environment.borrow_mut().get(&name.lexeme) {
                Some(value) => Ok(value.clone()),
                None => Err(Error::runtime_error(RuntimeError::R001, name.clone())),
            },
            Expr::Assign { name, expr } => {
                let val = self.visit_expression(expr)?;
                match self
                    .environment
                    .borrow_mut()
                    .assign(&name.lexeme, val.clone())
                {
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
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.visit_expression(&callee)?;
                match callee {
                    Value::Function(mut callee) => {
                        let args = self.prepare_func_args(callee.arity(), arguments, paren)?;
                        callee.call(self, args, paren)
                    }
                    Value::NativeFunction(mut callee) => {
                        let args = self.prepare_func_args(callee.arity(), arguments, paren)?;
                        callee.call(self, args, paren)
                    }
                    callee => Err(Error::type_error(TypeError::T004 { callee }, paren.clone())),
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
                self.environment.borrow_mut().define(&name.lexeme, value)
            }
            Stmt::Block { statements } => {
                {
                    self.environment.borrow_mut().nest()
                };
                for statement in statements {
                    self.visit_statement(&statement)?;
                }

                {
                    self.environment.borrow_mut().pop()
                };
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
                        .borrow_mut()
                        .define(&identifier.lexeme, Value::Number(num));
                    while num < dest {
                        self.environment
                            .borrow_mut()
                            .assign(&identifier.lexeme, Value::Number(num))
                            .unwrap();
                        self.visit_statement(&body)?;
                        num += step_by;
                    }
                }
            }
            Stmt::Function { name, params, body } => {
                let closure = {
                    Rc::new(RefCell::new(
                        self.environment.borrow_mut().clone().new_nested(),
                    ))
                };
                let function = Value::Function(Function {
                    name: name.clone(),
                    params: params
                        .iter()
                        .map(|p| p.lexeme.clone())
                        .collect::<Vec<String>>(),
                    body: body.clone(),
                    closure,
                });
                self.environment.borrow_mut().define(&name.lexeme, function)
            }
            Stmt::Return { keyword, expr } => {
                let value = match expr {
                    Some(expr) => self.visit_expression(expr)?,
                    None => Value::Null,
                };
                return Err(Error::return_error(value, keyword.clone()));
            }
        };
        Ok(())
    }
}
