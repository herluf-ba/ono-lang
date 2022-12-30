use std::{
    collections::{hash_map::Entry, HashMap},
    fmt,
};

use crate::interpreter::Value;

#[derive(Clone)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Value>,
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Environment")
            .field("enclosing", &self.enclosing)
            .field("values", &self.values)
            .finish()
    }
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn nest(&mut self) {
        let inner = Box::new(self.clone());
        *self = Self {
            enclosing: Some(inner),
            values: HashMap::new(),
        }
    }

    pub fn pop(&mut self) {
        let new_env = match &self.enclosing {
            None => None,
            Some(enclosing) => Some(enclosing.as_ref().clone()),
        };

        if let Some(new_env) = new_env {
            *self = new_env;
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&mut self, name: &str) -> Option<&Value> {
        match self.values.get(name) {
            Some(v) => Some(v),
            None => match &mut self.enclosing {
                Some(enclosing) => enclosing.get(name),
                None => None,
            },
        }
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), ()> {
        match self.values.entry(name.to_string()) {
            Entry::Occupied(mut entry) => {
                *entry.get_mut() = value;
                Ok(())
            }
            Entry::Vacant(_) => match &mut self.enclosing {
                Some(enclosing) => enclosing.assign(name, value),
                None => Err(()),
            },
        }
    }
}
