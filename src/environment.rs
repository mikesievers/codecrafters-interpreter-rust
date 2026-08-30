use std::collections::HashMap;

use crate::lox_value::LoxValue;

pub struct Environment<'a> {
    values: HashMap<&'a str, LoxValue>,
}

impl Environment<'_> {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }
}
