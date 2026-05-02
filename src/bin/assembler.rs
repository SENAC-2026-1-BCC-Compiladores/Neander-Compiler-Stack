use clap::Parser;
use neander_compiler_stack::assembler::{Lexer, ParserT};
use std::error::Error;
use std::fs;
use std::io;
use std::io::Read;

#[derive(Parser)]
struct Cli {
    #[arg(long, short)]
    path: Option<String>,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let data: String;
    let mut buff = String::new();

    // path para arquivo .asm
    if let Some(path) = cli.path {
        data = fs::read_to_string(path)?;
    } else {
        io::stdin().read_to_string(&mut buff)?;
        if buff.trim().is_empty() {
            return Err("Erro: Arquivo fornecido vazio ou inexistente".into());
        }

        data = buff;
    }

    let lexer = Lexer::new(&data);
    let mut parser = ParserT::new(lexer);
    parser.parse()?;

    Ok(())
}
