mod chunk;
mod compiler;
mod scanner;
mod value;
mod vm;

use std::{fs, ops::Add};

use qsv_docopt::Docopt;
use rprompt::prompt_reply;
use serde::Deserialize;

const USAGE: &'static str = "
Thorium virtual machine.

Usage:
    thorium
    thorium <path>
    thorium (-h | --help)
    thorium --version

Options:
    -h --help     Show this screen.
    --version     Show version.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_path: String,
    flag_version: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return ();
    }

    if args.arg_path.is_empty() {
        repl();
    } else {
        run_file(&args.arg_path);
    }

    // let mut chunk = chunk::Chunk::init();

    // let new_constant = chunk.add_constant(1.32);
    // chunk.write(chunk::OpCode::Constant.into(), 1);
    // chunk.write(new_constant as u8, 1);

    // let new_constant = chunk.add_constant(2.32);
    // chunk.write(chunk::OpCode::Constant.into(), 2);
    // chunk.write(new_constant as u8, 2);

    // let new_constant = chunk.add_constant(3.32);
    // chunk.write(chunk::OpCode::Constant.into(), 3);
    // chunk.write(new_constant as u8, 3);

    // chunk.write(chunk::OpCode::Negate.into(), 4);

    // chunk.write(chunk::OpCode::Add.into(), 5);

    // chunk.write(chunk::OpCode::Return.into(), 6);

    // chunk.disassemble("Chunk");

    // println!("\nStart VM");

    // let mut emulator = vm::Vm::init(&chunk);
    // let result = emulator.interpret();
    // match result {
    //     Ok(_) => println!("Program Successful."),
    //     Err(e) => println!("Program Error: {e}"),
    // }
}

fn repl() {
    loop {
        let line = prompt_reply("> ").unwrap();
        let line = line.add("\0");
        if line == "\n" {
            break;
        };
        compiler::compile(line);
        // let _ = vm::interpret(line);
    }
}

fn run_file(arg_path: &str) {
    let file_contents = fs::read_to_string(arg_path).expect("Failed to read file");
    let result = vm::interpret(file_contents);

    match result {
        Ok(()) => std::process::exit(0),
        Err(vm::InterpretError::CompileError) => std::process::exit(65),
        Err(vm::InterpretError::RuntimeError) => std::process::exit(70),
    };
}
