use std::collections::{hash_map::Entry, HashMap};

use crate::interpreter::Value;

pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&mut self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), ()> {
        match self.values.entry(name.to_string()) {
            Entry::Occupied(mut entry) => {
                *entry.get_mut() = value;
                Ok(())
            }
            Entry::Vacant(_) => Err(()),
        }
    }
}
