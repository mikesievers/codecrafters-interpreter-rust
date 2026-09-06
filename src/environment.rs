use std::collections::{HashMap, hash_map::Entry};

use crate::{lox_error::LoxError, lox_value::LoxValue};

pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Option<LoxValue>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new_enclosed(enclosing: Environment) -> Self {
        Environment {
            enclosing: Some(Box::new(enclosing)),
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
            Entry::Vacant(vacant) => Err(LoxError::RuntimeError(format!(
                "Unknown variable: {}",
                vacant.key()
            ))),
        }
    }

    pub fn get<T: AsRef<str>>(&self, name: &T) -> Result<Option<LoxValue>, LoxError> {
        match self.values.get(name.as_ref()) {
            Some(val) => Ok(val.clone()),
            None => match self.enclosing {
                Some(ref env) => Ok(env.get(name)?),
                None => Err(LoxError::RuntimeError(format!(
                    "Unknown variable: {}",
                    name.as_ref()
                ))),
            },
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

    #[test]
    fn test_enclosing() {
        // Define an outer environment and seed it
        let mut outer = Environment::new();
        let fourtytwo = LoxValue::Number(42.0);
        outer.put("a", Some(fourtytwo.clone()));
        outer.put("b", Some(fourtytwo.clone()));

        // Define an inner environment that shadows "a"
        let mut inner = Environment::new_enclosed(outer);
        let twentythree = LoxValue::Number(23.0);
        inner.put("a", Some(twentythree.clone()));

        // "a" is from the inner env, shadowed
        assert_eq!(inner.get(&"a").unwrap(), Some(twentythree.clone()));
        // "b" is from the outer env
        assert_eq!(inner.get(&"b").unwrap(), Some(fourtytwo.clone()));

        // Take the enclosing env out of the inner one and let the inner one expire
        // "a" should not be shadowed any more
        // "b" should be unchanged
        let outer = inner.enclosing.take().unwrap();
        assert_eq!(outer.get(&"a").unwrap(), Some(fourtytwo.clone()));
        assert_eq!(outer.get(&"b").unwrap(), Some(fourtytwo));
    }
}
