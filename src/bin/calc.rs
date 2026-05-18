use clap::Parser;
use neander_compiler_stack::calc::codegen::Codegen;
use neander_compiler_stack::calc::lexer::Lexer;
use neander_compiler_stack::calc::parser::CalcParser;
use std::error::Error;
use std::io::{self, Write};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    file: Option<String>,
}

fn process_expression(expr: &str) {
    println!("-> Compiling: {}", expr);
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let l = Lexer::new("14 + 13 + 2");
    let mut parser = CalcParser::new(l)?;
    let mut codegen = Codegen::new();
    let ast = parser.parse_expr()?;

    codegen.generate_code(&ast);
    println!("{}", codegen.code);

    let mut handle = io::stdout().lock();
    handle.write_all(codegen.code.as_bytes())?;
    handle.flush()?;

    Ok(())
}
