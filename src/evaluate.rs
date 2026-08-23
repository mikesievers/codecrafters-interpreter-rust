use crate::lox_value::LoxValue;
use crate::lox_error::LoxError;

pub trait Evaluate {
    fn evaluate(&self) -> Result<LoxValue, LoxError>;
}
