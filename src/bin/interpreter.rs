use clap::Parser;
use neander_compiler_stack::interpreter::Interpreter;
use std::error::Error;
use std::fs;
use std::io::{self, Read};

#[derive(Parser)]
struct Cli {
    #[arg(long, short)]
    path: Option<String>,

    #[arg(long, short)]
    pc: Option<u8>,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mut data: Vec<u8> = Vec::<u8>::new();

    if let Some(p) = cli.path {
        data = fs::read(p)?;
    } else {
        let mut handle = io::stdin().lock();
        handle.read_to_end(&mut data)?;
    }

    let mut interpreter = Interpreter::new();
    match interpreter.load_neander_file(&data) {
        Ok(interp) => interpreter = interp,
        Err(e) => {
            eprintln!("faild to load .MEM file: {}", e);
            return Ok(());
        }
    }

    if let Some(program_counter) = cli.pc {
        interpreter = Interpreter::set_pc(interpreter, program_counter);
    }
    interpreter.run();
    println!("{}", interpreter);

    Ok(())
}
