use std::{fmt::Display, ops::{Add, Div, Mul, Neg, Not, Sub}};

use crate::lox_error::LoxError;

// The LoxValue is almost identical to the TokenValue
// The exception is the String - while the TokenValue
// is expected not to duplicate String and refer to the
// program text instead, a LoxValue is expcted to hold
// e.g. a concatenation of Strings.
#[derive(Debug, Clone)]
pub enum LoxValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl Display for LoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            LoxValue::String(s) => { s.clone()},
            LoxValue::Number(n) => format!("{n}"),
            LoxValue::Boolean(b) => b.to_string(),
            LoxValue::Nil => "nil".to_string(),
        };
        write!(f, "{output}")
    }
}

impl Neg for LoxValue {
    type Output = Result<LoxValue, LoxError>;

    fn neg(self) -> Self::Output {
        match self {
            LoxValue::Number(n) => Ok(LoxValue::Number(-n)),
            _ => Err(LoxError::RuntimeError("Operand must be a number.".to_string())),
        }
    }
}

impl Not for LoxValue {
    type Output = LoxValue;

    fn not(self) -> Self::Output {
        match self {
            LoxValue::String(_) | LoxValue::Number(_) => LoxValue::Boolean(false),
            LoxValue::Boolean(b) => LoxValue::Boolean(!b),
            LoxValue::Nil => LoxValue::Boolean(true),
        }
    }
}

impl Sub for LoxValue {
    type Output = LoxValue;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (LoxValue::Number(l), LoxValue::Number(r)) => LoxValue::Number(l-r),
            _ => panic!("Trying to subtract incompatible value types"),
        }
    }
}

impl Mul for LoxValue {
    type Output = Result<LoxValue, LoxError>;

    fn mul(self, rhs: Self) -> Self::Output {
        match(self, rhs) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l*r)),
            _ => Err(LoxError::RuntimeError("Operands must be numbers.".to_string())),
        }
    }
}

impl Div for LoxValue {
    type Output = Result<LoxValue, LoxError>;
    
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l/r)),
            _ => Err(LoxError::RuntimeError("Operands must be numbers.".to_string())),
        }
    }
}

impl Add for LoxValue {
    type Output = LoxValue;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (LoxValue::String(l), LoxValue::String(r)) => LoxValue::String(l+&r),
            (LoxValue::Number(l), LoxValue::Number(r)) => LoxValue::Number(l+r),
            _ => panic!("Trying to add incompatible value types"),
        }
    }
}

impl PartialEq for LoxValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => *l0 == *r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl PartialOrd for LoxValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (LoxValue::Number(l), LoxValue::Number(r)) => l.partial_cmp(r),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lox_value::LoxValue;

    #[test]
    fn test_neg_number () {
        let n = LoxValue::Number(1.1);
        assert_eq!((-n).unwrap(), LoxValue::Number(-1.1));
    }

    #[test]
    fn test_not_truth() {
        let b = LoxValue::Boolean(true);
        assert_eq!(!b, LoxValue::Boolean(false));
    }
}