use crate::{
    ast::*,
    lexer::{Token, TokenKind},
};
use std::fmt::Display;

#[derive(Debug)]
pub enum Value {
    Error { message: String, token: Token },
    Bool(bool),
    Text(String),
    Number(f64),
    Null,
}

impl Value {
    fn is_equal(&self, other: &Value) -> bool {
        match self {
            Value::Error {
                message: _,
                token: _,
            } => false,
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
            Value::Error {
                message: _,
                token: _,
            } => "error",
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
            TokenKind::NULL => Value::Null,
            _ => Value::Error {
                token: token.clone(),
                message: "Unable to convert non-literal token to value".to_string(),
            },
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Error {
                message: _,
                token: _,
            } => Ok(()),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Text(v) => write!(f, "{}", v),
            Value::Number(v) => write!(f, "{}", v),
            Value::Null => write!(f, "null"),
        }
    }
}

pub struct Interpreter;

impl Interpreter {
    pub fn interpret(&mut self, expr: &Expr) -> Value {
        self.visit(expr)
    }

    fn error_from_token(token: &Token, message: &str) -> Value {
        Value::Error {
            token: token.clone(),
            message: message.to_string(),
        }
    }

    fn unary(operator: &Token, value: Value) -> Value {
        match operator.kind {
            TokenKind::BANG => match value {
                Value::Null => Value::Bool(false),
                Value::Bool(v) => Value::Bool(!v),
                _ => Interpreter::error_from_token(
                    operator,
                    &format!("Unable to negate value of type '{}'", value.display_type()),
                ),
            },
            TokenKind::MINUS => match value {
                Value::Number(v) => Value::Number(-v),
                _ => Interpreter::error_from_token(
                    operator,
                    &format!("Unable to negate value of type '{}'", value.display_type()),
                ),
            },
            _ => panic!(
                "Cannot interpret unary operator '{}'. The syntax tree is likely invalid",
                operator.lexeme
            ),
        }
    }

    fn binary(operator: &Token, left: Value, right: Value) -> Value {
        let mismatch_error = Interpreter::error_from_token(
            operator,
            &format!(
                "Cannot perform '{}' on a '{}' and a '{}'",
                operator.lexeme,
                left.display_type(),
                right.display_type()
            ),
        );

        match operator.kind {
            TokenKind::PLUS => match left {
                Value::Number(l) => match right {
                    Value::Number(r) => Value::Number(l + r),
                    _ => mismatch_error,
                },
                Value::Text(l) => match right {
                    Value::Text(r) => Value::Text(format!("{}{}", l, r)),
                    _ => mismatch_error,
                },
                _ => mismatch_error,
            },
            TokenKind::MINUS => match left {
                Value::Number(l) => match right {
                    Value::Number(r) => Value::Number(l - r),
                    _ => mismatch_error,
                },
                _ => mismatch_error,
            },
            TokenKind::STAR => match left {
                Value::Number(l) => match right {
                    Value::Number(r) => Value::Number(l * r),
                    _ => mismatch_error,
                },
                _ => mismatch_error,
            },
            TokenKind::SLASH => match left {
                Value::Number(l) => match right {
                    Value::Number(r) => {
                        if r != 0.0 {
                            Value::Number(l / r)
                        } else {
                            Interpreter::error_from_token(operator, "Division by zero")
                        }
                    }
                    _ => mismatch_error,
                },
                _ => mismatch_error,
            },
            TokenKind::GREATER => match left {
                Value::Number(l) => match right {
                    Value::Number(r) => Value::Bool(l > r),
                    _ => mismatch_error,
                },
                _ => mismatch_error,
            },
            TokenKind::GREATEREQUAL => match left {
                Value::Number(l) => match right {
                    Value::Number(r) => Value::Bool(l >= r),
                    _ => mismatch_error,
                },
                _ => mismatch_error,
            },
            TokenKind::EQUALEQUAL => Value::Bool(left.is_equal(&right)),
            TokenKind::BANGEQUAL => Value::Bool(!left.is_equal(&right)),
            TokenKind::LESS => match left {
                Value::Number(l) => match right {
                    Value::Number(r) => Value::Bool(l < r),
                    _ => mismatch_error,
                },
                _ => mismatch_error,
            },
            TokenKind::LESSEQUAL => match left {
                Value::Number(l) => match right {
                    Value::Number(r) => Value::Bool(l <= r),
                    _ => mismatch_error,
                },
                _ => mismatch_error,
            },
            _ => panic!(
                "Cannot interpret binary operator '{}'. The syntax tree is likely invalid",
                operator.lexeme
            ),
        }
    }
}

impl ExprVisitor<Value> for Interpreter {
    fn visit(&mut self, e: &Expr) -> Value {
        match e {
            Expr::Literal { value } => Value::from(value),
            Expr::Unary { operator, expr } => {
                let val = self.visit(expr);
                match val {
                    Value::Error {
                        message: _,
                        token: _,
                    } => val,
                    _ => Interpreter::unary(operator, val),
                }
            }
            Expr::Binary {
                operator,
                left,
                right,
            } => match self.visit(left) {
                Value::Error { message, token } => Value::Error { message, token },
                left => match self.visit(right) {
                    Value::Error { message, token } => Value::Error { message, token },
                    right => Interpreter::binary(operator, left, right),
                },
            },
            Expr::Group { expr } => self.visit(expr),
        }
    }
}
