use std::fmt::{Debug, Display};

use crate::{interpreter::Interpreter, lox_error::LoxError, lox_value::LoxValue};

pub trait Callable: Debug + Display {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, LoxError>;
}
