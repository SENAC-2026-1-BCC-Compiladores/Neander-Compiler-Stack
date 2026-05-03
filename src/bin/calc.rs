use clap::Parser;
use neander_compiler_stack::calc::lexer::Lexer;
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
    let mut l = Lexer::new("14 + 13");

    while let Some(t) = l.next_token() {
        match t {
            Ok(token) => println!("{:?}", token),
            Err(_) => return Err("erro".into()),
        }
    }

    Ok(())
}
