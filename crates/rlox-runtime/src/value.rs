use rlox_intermediate::Constant;

#[derive(Debug)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Nil,
}

impl From<Constant> for Value {
    fn from(value: Constant) -> Self {
        match value {
            Constant::Number(number) => Value::Number(number),
            _ => unimplemented!(),
        }
    }
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
            _ => false,
        }
    }
}
