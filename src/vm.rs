use crate::{
    chunk::{display, Chunk, OpCode},
    compiler,
    value::{Value, ValueArray},
};
use colored::Colorize;
use custom_error::custom_error;

pub struct Vm<'a> {
    chunk: &'a Chunk,
    ip: usize,
    stack: ValueArray,
}

custom_error! { pub InterpretError
    CompileError = "Error during compilation.",
    RuntimeError = "Error during execution",
}

impl<'a> Vm<'a> {
    pub fn init(chunk: &'a Chunk) -> Self {
        Vm {
            chunk,
            ip: 0,
            stack: ValueArray::init(),
        }
    }

    pub fn interpret(&mut self) -> Result<(), InterpretError> {
        if cfg!(debug_assertions) {
            println!(
                "{}",
                "\nVM Running\n".to_string().magenta().underline().bold()
            );
        }

        while self.ip < self.chunk.code.len() {
            let instruction = self.read_instruction()?;

            if cfg!(debug_assertions) {
                println!("{}", format!("{} top", self.stack).truecolor(234, 142, 68));
                display(self.chunk, Some(&instruction), self.ip - 1, "");
            }

            match instruction {
                OpCode::Return => {
                    let value = self.stack.pop();
                    return match value {
                        Some(v) => {
                            println!("{v}");
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
                        Some(v) => {
                            if let Value::Number(n) = v {
                                self.stack.push(Value::Number(-n))
                            } else {
                                self.runtime_error("Operand must be a number");
                                return Err(InterpretError::RuntimeError);
                            }
                        }
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
                OpCode::True => self.stack.push(Value::Boolean(true)),
                OpCode::False => self.stack.push(Value::Boolean(false)),
                OpCode::Nil => self.stack.push(Value::Nil),
                OpCode::Not => {
                    let v = self.stack.pop();
                    if let Some(v) = v {
                        match v {
                            Value::Boolean(b) => self.stack.push(Value::Boolean(!b)),
                            Value::Nil => self.stack.push(Value::Boolean(true)),
                            _ => self.stack.push(Value::Boolean(false)),
                        }
                    } else {
                        self.runtime_error("Stack underflow");
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OpCode::Equal => {
                    let value = self.equal_op();
                    if let Ok(value) = value {
                        self.stack.push(value);
                    }
                }
                OpCode::Greater => {
                    let value = self.compare_op(|a, b| a > b);
                    if let Ok(value) = value {
                        self.stack.push(value);
                    }
                }
                OpCode::Less => {
                    let value = self.compare_op(|a, b| a < b);
                    if let Ok(value) = value {
                        self.stack.push(value);
                    }
                }
            }
        }

        Ok(())
    }

    fn equal_op(&mut self) -> Result<Value, InterpretError> {
        let b = self.stack.pop();
        let a = self.stack.pop();

        if let (Some(a), Some(b)) = (a, b) {
            Ok(Value::Boolean(a == b))
        } else {
            Err(InterpretError::RuntimeError)
        }
    }

    fn compare_op<F: Fn(Value, Value) -> bool>(&mut self, op: F) -> Result<Value, InterpretError> {
        let b = self.stack.pop();
        let a = self.stack.pop();

        if let (Some(a), Some(b)) = (a, b) {
            Ok(Value::Boolean(op(a, b)))
        } else {
            Err(InterpretError::RuntimeError)
        }
    }

    fn binary_op<F: Fn(f32, f32) -> f32>(&mut self, op: F) -> Result<Value, InterpretError> {
        let b = self.stack.pop();
        let a = self.stack.pop();

        if let (Some(Value::Number(a)), Some(Value::Number(b))) = (a, b) {
            Ok(Value::Number(op(a, b)))
        } else {
            self.runtime_error("Operand must be a number");
            Err(InterpretError::RuntimeError)
        }
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.code[self.ip];
        self.ip += 1;
        byte
    }

    fn read_constant(&mut self) -> Value {
        let index = self.read_byte();

        self.chunk.constants.values[index as usize]
    }

    fn read_instruction(&mut self) -> Result<OpCode, InterpretError> {
        let byte = self.read_byte();
        let instruction = OpCode::try_from(byte);
        match instruction {
            Ok(value) => Ok(value),
            Err(_) => Err(InterpretError::RuntimeError),
        }
    }

    fn runtime_error(&mut self, arg: &str) {
        eprintln!("{}", arg);

        let line = self.chunk.lines[self.ip - 1];
        eprintln!("[line {line}] in script");
        self.stack.reset();
    }
}

pub fn interpret(source: String) -> Result<(), InterpretError> {
    let chunk = compiler::compile(source)?;
    let mut vm = Vm::init(&chunk);
    vm.interpret()?;
    Ok(())
}
