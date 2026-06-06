use clap::Parser;
use neander_compiler_stack::interpreter::Interpreter;
use std::error::Error;
use std::fs;
use std::io::{self, Read};

#[derive(Parser)]
#[command(
    name = "interpreter",
    version = "0.1.0",
    about = "A virtual machine for the NEANDER architecture",
    long_about = "\
Loads a .MEM file specified by the '--path' option.

If no path is provided, the program reads from standard input (stdin).

The '--pc' option can be used to specify the initial program counter value.
"
)]
struct Cli {
    /// Path to the .MEM file to be read
    #[arg(long, short)]
    path: Option<String>,

    /// Initial program counter value
    #[arg(long)]
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
