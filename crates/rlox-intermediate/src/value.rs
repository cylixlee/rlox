use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::Deref;

use crate::Function;
use crate::Reference;

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(Reference<String>),
    Function(Reference<Function>),
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
            (Value::Function(this), Value::Function(that)) => this == that,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
        match self {
            Value::Nil => { /* every Nil value is the same. */ }
            Value::Boolean(boolean) => boolean.hash(state),
            Value::Number(number) => number.to_bits().hash(state),
            Value::String(string) => string.hash(state),
            Value::Function(function) => function.hash(state),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(boolean) => write!(f, "{boolean}"),
            Value::Number(number) => write!(f, "{number}"),
            Value::String(string) => write!(f, "{}", string.deref()),
            Value::Function(function) => write!(f, "<fun {}>", function.name()),
        }
    }
}
