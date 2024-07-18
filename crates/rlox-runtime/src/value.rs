use std::fmt::{Display, Formatter};
use std::ops::Deref;

use crate::heap::Reference;

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(Reference<String>),
}

impl Value {
    pub fn boolean(&self) -> bool {
        match self {
            Value::Boolean(boolean) => *boolean,
            Value::Nil => false,
            _ => true,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Boolean(this), Value::Boolean(that)) => this == that,
            (Value::Number(this), Value::Number(that)) => (this - that).abs() < f64::EPSILON,
            (Value::String(this), Value::String(that)) => {
                if this == that {
                    return true;
                }
                this.deref() == that.deref()
            }
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(boolean) => write!(f, "{boolean}"),
            Value::Number(number) => write!(f, "{number}"),
            Value::String(string) => write!(f, "{}", string.deref()),
        }
    }
}
