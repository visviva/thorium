use crate::value::{Value, ValueArray};
use colored::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryFrom;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum OpCode {
    Return,
    Constant,
    Negate,
    Add,
    Subtract,
    Divide,
    Multiply,
    True,
    False,
    Nil,
    Not,
    Equal,
    Greater,
    Less,
    Print,
    Pop,
    DefineGlobal,
    GetGlobal,
    SetGlobal,
}

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: ValueArray,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn init() -> Chunk {
        Chunk {
            code: Vec::new(),
            constants: ValueArray::init(),
            lines: Vec::new(),
        }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.write(value);
        self.constants.values.len() - 1
    }

    pub fn disassemble(&self, name: &str) {
        println!(
            "{}",
            format!("\nDisassemble {name}\n")
                .magenta()
                .bold()
                .underline()
        );

        let mut offset = 0;

        while offset < self.code.len() {
            let op = self.code[offset];
            let op_code = OpCode::try_from(op);

            if let Err(value) = op_code {
                display(self, None, offset, &format!("{}", value.number));
                offset += 1;
                continue;
            }

            if let Ok(code) = op_code {
                match code {
                    OpCode::Return
                    | OpCode::Negate
                    | OpCode::Add
                    | OpCode::Subtract
                    | OpCode::Divide
                    | OpCode::Multiply
                    | OpCode::True
                    | OpCode::False
                    | OpCode::Nil
                    | OpCode::Not
                    | OpCode::Equal
                    | OpCode::Greater
                    | OpCode::Less
                    | OpCode::Print
                    | OpCode::Pop => offset = display_simple_instruction(&code, offset, self),

                    OpCode::Constant
                    | OpCode::DefineGlobal
                    | OpCode::GetGlobal
                    | OpCode::SetGlobal => {
                        offset = display_constant_instruction(&code, offset, self)
                    }
                }
            }
        }
    }
}

pub fn display(chunk: &Chunk, op: Option<&OpCode>, offset: usize, data: &str) {
    println!(
        "{:0>4}\t{}\t{} {}",
        format!("{:?}", offset).green(),
        if (offset > 0) && (chunk.lines[offset] == chunk.lines[offset - 1]) {
            "|".to_string()
        } else {
            format!("{}", chunk.lines[offset])
        },
        if let Some(mnemonic) = op {
            format!("{:?}", mnemonic).blue().bold()
        } else {
            "Unknown OP".to_string().red().bold()
        },
        data
    );
}

fn display_simple_instruction(op: &OpCode, offset: usize, chunk: &Chunk) -> usize {
    display(chunk, Some(op), offset, "");
    offset + 1
}

fn display_constant_instruction(op: &OpCode, offset: usize, chunk: &Chunk) -> usize {
    let constant_index = chunk.code[offset + 1];
    let constant_value = chunk.constants.values[constant_index as usize].clone();
    display(
        chunk,
        Some(op),
        offset,
        &format!("Index={constant_index} Value={constant_value}"),
    );
    offset + 2
}
