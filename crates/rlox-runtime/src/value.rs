use rlox_intermediate::Constant;

#[derive(Debug)]
pub enum Value {
    Number(f64),
    String(String),
}

impl From<Constant> for Value {
    fn from(value: Constant) -> Self {
        match value {
            Constant::Number(number) => Value::Number(number),
            Constant::String(string) => Value::String(string),
        }
    }
}
