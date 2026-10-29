use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    lox_error::LoxError::{self, RuntimeError},
    lox_value::LoxValue,
};

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.as_str())
    }
}

impl Class {
    pub fn new(name: &String) -> Self {
        Class { name: name.clone() }
    }
}

#[derive(Debug)]
pub struct Instance {
    pub class: Rc<Class>,
    fields: HashMap<String, LoxValue>,
}

impl Instance {
    pub fn new(class: Rc<Class>) -> Self {
        Instance {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Result<LoxValue, LoxError> {
        self.fields
            .get(name)
            .cloned()
            .ok_or_else(|| LoxError::RuntimeError(format!("Undefined property '{name}'.")))
    }
}
