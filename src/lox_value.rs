use std::{fmt::Display, ops::{Mul, Neg, Not, Sub}};

// The LoxValue is almost identical to the TokenValue
// The exception is the String - while the TokenValue
// is expected not to duplicate String and refer to the
// program text instead, a LoxValue is expcted to hold
// e.g. a concatenation of Strings.
#[derive(Debug, Clone, PartialEq)]
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
    type Output = LoxValue;

    fn neg(self) -> Self::Output {
        match self {
            LoxValue::String(s) => todo!(),
            LoxValue::Number(n) => LoxValue::Number(-n),
            LoxValue::Boolean(_) => todo!(),
            LoxValue::Nil => todo!(),
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
    type Output = LoxValue;

    fn mul(self, rhs: Self) -> Self::Output {
        match(self, rhs) {
            (LoxValue::Number(l), LoxValue::Number(r)) => LoxValue::Number(l*r),
            _ => panic!("Trying to multiply incompatible value types"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lox_value::LoxValue;

    #[test]
    fn test_neg_number () {
        let n = LoxValue::Number(1.1);
        assert_eq!(-n, LoxValue::Number(-1.1));
    }

    #[test]
    fn test_not_truth() {
        let b = LoxValue::Boolean(true);
        assert_eq!(!b, LoxValue::Boolean(false));
    }
}