use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::Entry},
    rc::Rc,
};

use crate::{lox_error::LoxError, lox_value::LoxValue};

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Clone, Debug)]
pub struct Environment {
    enclosing: Option<EnvRef>,
    // NOTE: values are exposed as pub to allow for the tests.
    // Usually this should be avoided, e.g. by wrapping get() of the values directly
    pub values: HashMap<String, Option<LoxValue>>,
}

// To access enclosing environments ("ancestors"), define an extensiontrai
// It is defined on Option<EnvRef> to also enable access the environment itself
// (which cannot reference itself in terms of an Option<>)
pub trait EnvironmentExt {
    fn ancestor(&self, distance: usize) -> EnvRef;
}

impl EnvironmentExt for EnvRef {
    fn ancestor(&self, distance: usize) -> EnvRef {
        // Distance 0, Start with the current environment
        let mut ancestor = self.clone();

        // Distance 1+: Find the ancestor at <distance>
        // If the distance is 0, the loop is skipped
        for _ in 1..=distance {
            let next_ancestor = ancestor
                .borrow()
                .enclosing
                .as_ref()
                .expect("Distance is too far when finding an ancestor.")
                .clone();
            ancestor = next_ancestor;
        }

        ancestor
    }
}

impl Environment {
    pub fn new_global() -> EnvRef {
        Rc::new(RefCell::new(Environment {
            enclosing: None,
            values: HashMap::new(),
        }))
    }

    pub fn new_child(enclosing: &EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Environment {
            enclosing: Some(enclosing.clone()),
            values: HashMap::new(),
        }))
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

        match self.values.entry(name_str.clone()) {
            Entry::Occupied(mut entry) => {
                *entry.get_mut() = value;
                Ok(())
            }
            Entry::Vacant(vacant) => match self.enclosing {
                Some(ref mut env) => env.borrow_mut().assign(name_str, value),
                None => Err(LoxError::RuntimeError(format!(
                    "Unknown variable: {}",
                    vacant.key()
                ))),
            },
        }
    }

    pub fn get<T: AsRef<str>>(&self, name: &T) -> Result<Option<LoxValue>, LoxError> {
        match self.values.get(name.as_ref()) {
            Some(val) => Ok(val.clone()),
            None => match self.enclosing {
                Some(ref env) => Ok(env.borrow().get(name)?),
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
        let env = Environment::new_global();

        let fourtytwo = LoxValue::Number(42.0);

        env.borrow_mut().put("a", Some(fourtytwo.clone()));
        assert_eq!(env.borrow().get(&"a").unwrap(), Some(fourtytwo));
    }

    #[test]
    fn test_enclosingd() {
        // Define an outer environment and seed it
        let outer = Environment::new_global();
        let fourtytwo = LoxValue::Number(42.0);
        outer.borrow_mut().put("a", Some(fourtytwo.clone()));
        outer.borrow_mut().put("b", Some(fourtytwo.clone()));
        outer.borrow_mut().put("c", Some(fourtytwo.clone()));

        // Define an inner environment that shadows "a"
        let inner = Environment::new_child(&outer);
        let twentythree = LoxValue::Number(23.0);
        inner.borrow_mut().put("a", Some(twentythree.clone()));
        // Write into "c", which only exists in the enclosing env
        let one = LoxValue::Number(1.0);
        inner.borrow_mut().assign("c", Some(one.clone())).unwrap();

        // "a" is from the inner env, shadowed
        assert_eq!(inner.borrow().get(&"a").unwrap(), Some(twentythree.clone()));
        // "b" is from the outer env
        assert_eq!(inner.borrow().get(&"b").unwrap(), Some(fourtytwo.clone()));

        // Take the enclosing env out of the inner one and let the inner one expire
        // "a" should not be shadowed any more
        // "b" should be unchanged
        let outer = inner
            .borrow_mut()
            .enclosing
            .take()
            .expect("Inner environment must have an enclosing env");
        assert_eq!(outer.borrow().get(&"a").unwrap(), Some(fourtytwo.clone()));
        assert_eq!(outer.borrow().get(&"b").unwrap(), Some(fourtytwo));
        // "c" should have been overwritten through the inner env
        assert_eq!(outer.borrow().get(&"c").unwrap(), Some(one));
    }

    #[test]
    fn test_ancestor() {
        // Define an outer environment and seed it
        let outer = Environment::new_global();
        let fourtytwo = LoxValue::Number(42.0);
        outer.borrow_mut().put("a", Some(fourtytwo.clone()));

        // Define an empty middle environment
        let middle = Environment::new_child(&outer);

        // Define an inner environment that shadows "a"
        let inner = Environment::new_child(&middle);
        let twentythree = LoxValue::Number(23.0);
        inner.borrow_mut().put("a", Some(twentythree.clone()));

        // the inner env is ancestor 0, it should hold 23
        // Look directly into the HashMap to circumvent resolution logic
        assert_eq!(
            *inner.ancestor(0).as_ref().borrow().values.get("a").unwrap(),
            Some(LoxValue::Number(23.0))
        );

        // the middle env is ancestor 1, it should be empty
        assert_eq!(inner.ancestor(1).as_ref().borrow().values.get("a"), None);

        // the outer env is ancestor 2, it should hold 42.0
        assert_eq!(
            *inner.ancestor(2).as_ref().borrow().values.get("a").unwrap(),
            Some(LoxValue::Number(42.0))
        );
    }
}
