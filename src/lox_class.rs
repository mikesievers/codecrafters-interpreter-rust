use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    lox_error::LoxError::{self, RuntimeError},
    lox_value::LoxValue,
};

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    methods: HashMap<String, LoxValue>,
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.as_str())
    }
}

impl Class {
    pub fn new(name: &str, methods: HashMap<String, LoxValue>) -> Self {
        Class {
            name: name.to_owned(),
            methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<LoxValue> {
        self.methods.get(name).cloned()
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

    pub fn has_field(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Result<LoxValue, LoxError> {
        // Return field, if found
        if self.fields.contains_key(name) {
            return self.fields.get(name).cloned().ok_or_else(|| {
                LoxError::RuntimeError("Trying to return an undefined field".into())
            });
        }
        // Return method, if found
        if let Some(method) = self.class.find_method(name) {
            return Ok(method);
        }

        Err(LoxError::RuntimeError(format!(
            "Undefined property '{name}'."
        )))
    }

    pub fn set(&mut self, name: &str, value: LoxValue) {
        self.fields.insert(name.to_string(), value);
    }
}
