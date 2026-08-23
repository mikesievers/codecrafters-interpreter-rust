use crate::lox_value::LoxValue;

pub trait Evaluate {
    fn evaluate(&self) -> LoxValue;
}
