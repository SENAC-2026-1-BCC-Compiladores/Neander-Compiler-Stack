use clap::Parser;
use neander_compiler_stack::calc::lexer::Lexer;
use neander_compiler_stack::calc::parser::CalcParser;
use std::error::Error;

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
    let ast = parser.parse_expr()?;
    ast.print(0);

    Ok(())
}
