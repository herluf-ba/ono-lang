use std::{
    collections::{hash_map::Entry, HashMap},
    fmt,
};

#[derive(Clone)]
pub struct Environment<V: Clone + std::fmt::Debug> {
    enclosing: Option<Box<Environment<V>>>,
    values: HashMap<String, V>,
}

impl<V: Clone + std::fmt::Debug> fmt::Debug for Environment<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Environment")
            .field("enclosing", &self.enclosing)
            .field("values", &self.values)
            .finish()
    }
}

impl<V: Clone + std::fmt::Debug> Environment<V> {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new_nested(&self) -> Self {
        let inner = Box::new(self.clone());
        Self {
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

    pub fn define(&mut self, name: &str, value: V) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&mut self, name: &str) -> Option<&V> {
        match self.values.get(name) {
            Some(v) => Some(v),
            None => match &mut self.enclosing {
                Some(enclosing) => enclosing.get(name),
                None => None,
            },
        }
    }

    pub fn assign(&mut self, name: &str, value: V) -> Result<(), ()> {
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
