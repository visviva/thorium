use std::{cmp::Ordering, fmt};

#[derive(Clone, Copy)]
pub enum Value {
    Boolean(bool),
    Nil,
    Number(f32),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Number(n) => write!(f, "{}", n),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => a.partial_cmp(b),
            (Value::Nil, Value::Nil) => Some(Ordering::Equal),
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

pub struct ValueArray {
    pub values: Vec<Value>,
}

impl ValueArray {
    pub fn init() -> Self {
        Self { values: Vec::new() }
    }

    pub fn write(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn push(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.values.pop()
    }

    pub fn reset(&mut self) {
        self.values.clear()
    }

    pub fn peek(&self, distance: usize) -> Option<&Value> {
        let index = self.values.len() - 1 - distance;
        Some(&self.values[index])
    }
}

impl fmt::Display for ValueArray {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::new();
        output += "[";
        for value in &self.values {
            output.push_str(&format!("{}, ", value));
        }
        output += "]";
        write!(f, "{}", output)
    }
}
