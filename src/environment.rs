use std::collections::{HashMap, hash_map::Entry};

use crate::{lox_error::LoxError, lox_value::LoxValue};

pub struct Environment {
    values: HashMap<String, Option<LoxValue>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn put<T: Into<String>>(&mut self, name: T, value: Option<LoxValue>) {
        self.values.insert(name.into(), value);
    }

    pub fn assign<T: Into<String>>(
        &mut self,
        name: T,
        value: Option<LoxValue>,
    ) -> Result<(), LoxError> {
        let name_str: String = name.into();

        match self.values.entry(name_str) {
            Entry::Occupied(mut entry) => {
                *entry.get_mut() = value;
                Ok(())
            }
            Entry::Vacant(vacant) => Err(LoxError::RuntimeError(
                format!("Unknown variable: {}", vacant.key()),
            )),
        }
    }

    pub fn get<T: AsRef<str>>(&self, name: &T) -> Result<Option<LoxValue>, LoxError> {
        match self.values.get(name.as_ref()) {
            Some(val) => Ok(val.clone()),
            None => Err(LoxError::RuntimeError(format!(
                "Unknown variable: {}",
                name.as_ref()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment() {
        let mut env = Environment::new();

        let fourtytwo = LoxValue::Number(42.0);

        env.put("a", Some(fourtytwo.clone()));
        assert_eq!(env.get(&"a").unwrap(), Some(fourtytwo));
    }
}
