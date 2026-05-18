use clap::Parser;
use neander_compiler_stack::assembler::Codegen;
use neander_compiler_stack::assembler::{Lexer, ParserT};
use std::error::Error;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    #[arg(long, short)]
    path: Option<String>,

    #[arg(long, short)]
    output: Option<String>,
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

    let mut parser = ParserT::new(Lexer::new(&data));
    let mut codegen = Codegen::new();
    let program = parser.parse()?;
    let bin = codegen.generate(&program)?;

    if let Some(path) = cli.output {
        let mut p = PathBuf::from(path);

        if p.is_dir() {
            p.push("neander_out.MEM");
        } else if p.extension().is_none() {
            p.set_extension(".MEM");
        }

        fs::write(&p, bin)?;
        eprintln!("File saved at: {}", p.display());
    } else {
        let mut handle = io::stdout().lock();
        handle.write_all(&bin)?;
        handle.flush()?;
    }

    Ok(())
}
