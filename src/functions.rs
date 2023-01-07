use crate::{
    ast::{Stmt, StmtVisitor},
    environment::Environment,
    error::{Error, ErrorKind, RuntimeError},
    interpreter::{Interpreter, Value},
    lexer::{Token, TokenKind},
};

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Token,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub closure: Environment,
}

pub trait Func {
    fn arity(&self) -> usize;
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
        called_at: &Token,
    ) -> Result<Value, Error>;
}

impl Func for Function {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
        _called_at: &Token,
    ) -> Result<Value, Error> {
        let prev_env = interpreter.environment.clone();
        interpreter.environment = self.closure.clone();

        for (i, arg) in arguments.iter().enumerate() {
            interpreter.environment.define(&self.params[i], arg.clone());
        }

        let mut return_value = Value::Null;
        for stmt in self.body.iter() {
            if let Err(error) = interpreter.visit_statement(&stmt) {
                match error.kind {
                    ErrorKind::Return(value) => {
                        return_value = value;
                        break;
                    }
                    _ => return Err(error),
                };
            }
        }
        self.closure = interpreter.environment.clone();
        interpreter.environment = prev_env;
        Ok(return_value)
    }
}

type NativeFunctionDef = dyn Fn(Vec<Value>) -> Result<Value, Error>;

#[derive(Clone)]
pub struct NativeFunction {
    pub name: &'static str,
    pub params: Vec<String>,
    pub func: &'static NativeFunctionDef,
}

impl NativeFunction {
    pub fn new(name: &'static str, func: &'static NativeFunctionDef) -> Self {
        Self {
            name,
            func,
            params: Vec::new(),
        }
    }
}

impl std::fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NativeFunction<{}>", self.name)
    }
}

impl Func for NativeFunction {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(
        &mut self,
        _interpreter: &mut Interpreter,
        arguments: Vec<Value>,
        called_at: &Token,
    ) -> Result<Value, Error> {
        match (self.func)(arguments) {
            Ok(val) => Ok(val),
            Err(mut error) => {
                error.token = called_at.clone();
                Err(error)
            }
        }
    }
}

fn nf_clock(_arguments: Vec<Value>) -> Result<Value, Error> {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(n) => Ok(Value::Number(n.as_nanos() as f64)),
        Err(_) => Err(Error {
            file: None,
            line_src: None,
            token: Token {
                kind: TokenKind::NULL,
                lexeme: "".to_string(),
                row: 0,
                column: 0,
            },
            kind: ErrorKind::Runtime(RuntimeError::R004),
        }),
    }
}

pub fn create_global_environment() -> Environment {
    let mut env = Environment::new();
    env.define(
        "clock",
        Value::NativeFunction(NativeFunction::new("clock", &nf_clock)),
    );
    env
}
