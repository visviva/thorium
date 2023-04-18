pub type Value = f32;

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
}
