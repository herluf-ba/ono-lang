use std::collections::{hash_map::Entry, HashMap};

use crate::interpreter::Value;

#[derive(Clone)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new_nested(parent: &Environment) -> Self {
        Self {
            enclosing: Some(Box::new(parent.clone())),
            values: HashMap::new(),
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
