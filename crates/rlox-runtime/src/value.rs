use crate::heap::Reference;

#[derive(Debug)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(Reference<String>),
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        match value {
            Value::Boolean(boolean) => boolean,
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
                &**this == &**that
            }
            _ => false,
        }
    }
}
