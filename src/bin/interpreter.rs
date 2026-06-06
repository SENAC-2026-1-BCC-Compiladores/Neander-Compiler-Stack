use clap::Parser;
use neander_compiler_stack::interpreter::Interpreter;
use std::error::Error;
use std::fs;
use std::io::{self, Read};

#[derive(Parser)]
#[command(
    name = "interpreter",
    version = "0.1.0",
    about = "Máquina virtual para interpretar arquivos .MEM para a arquitetura NEANDER",
    long_about = "\
Carrega um arquivo .MEM indicado pela opção '--path'.

Caso nenhum caminho seja informado, o programa utiliza a entrada padrão (stdin).

O usuário também pode usar a opção '--pc' para definir o contador inicial de instruções.
"
)]
struct Cli {
    /// Path do arquivo .MEM a ser lido
    #[arg(long, short)]
    path: Option<String>,

    /// Contador inicial do programa
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
