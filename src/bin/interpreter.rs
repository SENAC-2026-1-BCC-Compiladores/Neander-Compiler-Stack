use clap::Parser;
use neander_compiler_stack::interpreter::Interpreter;
use std::error::Error;
use std::fs;
use std::io;

#[derive(Parser)]
struct Cli {
    #[arg(long, short)]
    path: Option<String>,

    #[arg(long, short)]
    pc: Option<u8>,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let data: Vec<u8>;
    let mut buff = String::new();

    if let Some(p) = cli.path {
        data = fs::read(p)?;
    } else {
        io::stdin().read_line(&mut buff)?;
        let file_name = buff.trim().replace("\"", "");
        data = fs::read(file_name)?;
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
