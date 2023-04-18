use crate::{
    chunk::{display, Chunk, OpCode},
    compiler,
    value::Value,
};
use colored::Colorize;
use custom_error::custom_error;

pub struct Vm<'a> {
    chunk: &'a Chunk,
    ip: usize,
    stack: Vec<Value>,
}

custom_error! { pub InterpretError
    CompileError = "Error during compilation.",
    RuntimeError = "Error during execution",
}

impl<'a> Vm<'a> {
    pub fn init(chunk: &'a Chunk) -> Self {
        Vm {
            chunk: chunk,
            ip: 0,
            stack: Vec::new(),
        }
    }

    pub fn interpret(&mut self) -> Result<(), InterpretError> {
        while self.ip < self.chunk.code.len() {
            let instruction = self.read_instruction()?;

            if cfg!(debug_assertions) {
                println!(
                    "{}",
                    format!("{:?} top", self.stack).truecolor(234, 142, 68)
                );
                display(self.chunk, Some(&instruction), self.ip - 1, "");
            }

            match instruction {
                OpCode::Return => {
                    let value = self.stack.pop();
                    return match value {
                        Some(v) => {
                            println!("{v:?}");
                            return Ok(());
                        }
                        None => {
                            println!("Stack Underflow");
                            Err(InterpretError::RuntimeError)
                        }
                    };
                }

                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.stack.push(constant);
                }

                OpCode::Negate => {
                    let value = self.stack.pop();
                    match value {
                        Some(v) => self.stack.push(-v),
                        None => {
                            println!("Stack Underflow");
                            return Err(InterpretError::RuntimeError);
                        }
                    }
                }

                OpCode::Add => match self.binary_op(|a, b| a + b) {
                    Ok(v) => self.stack.push(v),
                    Err(_) => return Err(InterpretError::RuntimeError),
                },
                OpCode::Subtract => match self.binary_op(|a, b| a - b) {
                    Ok(v) => self.stack.push(v),
                    Err(_) => return Err(InterpretError::RuntimeError),
                },
                OpCode::Divide => match self.binary_op(|a, b| a / b) {
                    Ok(v) => self.stack.push(v),
                    Err(_) => return Err(InterpretError::RuntimeError),
                },
                OpCode::Multiply => {
                    let value = self.binary_op(|a, b| a * b)?;
                    self.stack.push(value);
                }
            }
        }

        Ok(())
    }

    fn binary_op<F: Fn(Value, Value) -> Value>(&mut self, op: F) -> Result<Value, InterpretError> {
        let b = self.stack.pop();

        if let None = b {
            return Err(InterpretError::RuntimeError);
        }

        let a = self.stack.pop();

        if let None = a {
            return Err(InterpretError::RuntimeError);
        }

        Ok(op(a.unwrap(), b.unwrap()))
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.code[self.ip];
        self.ip += 1;
        byte
    }

    fn read_constant(&mut self) -> Value {
        let index = self.read_byte();
        let constant = self.chunk.constants.values[index as usize];
        constant
    }

    fn read_instruction(&mut self) -> Result<OpCode, InterpretError> {
        let byte = self.read_byte();
        let instruction = OpCode::try_from(byte);
        match instruction {
            Ok(value) => Ok(value),
            Err(_) => Err(InterpretError::RuntimeError),
        }
    }
}

pub fn interpret(source: String) -> Result<(), InterpretError> {
    compiler::compile(source);
    return Ok(());
}
