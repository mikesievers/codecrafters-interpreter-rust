use crate::{interpreter::Interpreter, lox_error::LoxError, lox_value::LoxValue};

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, LoxError>;
}
