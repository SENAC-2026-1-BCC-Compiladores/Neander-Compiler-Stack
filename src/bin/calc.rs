use clap::Parser;
use neander_compiler_stack::assembler::{Codegen, Lexer, ParserT};
use neander_compiler_stack::calc::codegen::CodegenCalc;
use neander_compiler_stack::calc::lexer::LexerCalc;
use neander_compiler_stack::calc::parser::CalcParser;
use neander_compiler_stack::interpreter::Interpreter;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "calc",
    version = "0.1.0",
    about = "A calculator for the NEANDER architecture",
    long_about = "\
A calculator that can be used from the command line to perform addition,
subtraction, and multiplication operations.

It can also read a .txt file containing arithmetic expressions and generate
equivalent NEANDER assembly code compatible with the assembler from the
Neander Compiler Stack.
"
)]
struct Cli {
    /// Path to the .txt file to be read
    #[arg(short, long)]
    path: Option<String>,

    /// Path to the .asm file to be written
    #[arg(short, long)]
    output: Option<String>,
}
fn process_expression() {
    println!("Neander interactive Calculator - Type 'q' to quit");
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("calc> ");

        if stdout.flush().is_err() {
            break;
        }

        let mut input = String::new();
        if stdin.read_line(&mut input).is_err() {
            break;
        }

        let expr = input.trim();
        if expr == "q" || expr == "quit" {
            println!("Exiting...");
            break;
        }

        if expr.is_empty() {
            continue;
        }

        let calc_lexer = LexerCalc::new_with_stream(expr);
        let mut calc_parser = match CalcParser::new(calc_lexer) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Lexer error: {}", e);
                continue;
            }
        };

        let ast = match calc_parser.parse_expr() {
            Ok(tree) => tree,
            Err(e) => {
                eprintln!("Syntax error: {}", e);
                continue;
            }
        };

        let mut codegen = CodegenCalc::new();
        codegen.generate_code(&ast);
        let asm_text = &codegen.code;

        let assembler_lexer = Lexer::new(asm_text);
        let mut assembler_parser = ParserT::new(assembler_lexer);

        let p = match assembler_parser.parse() {
            Ok(prog) => prog,
            Err(e) => {
                eprintln!("Error on assembler: {}", e);
                continue;
            }
        };

        let mut assembler_codegen = Codegen::new();
        let bin = match assembler_codegen.generate(&p) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error generating binary: {}", e);
                continue;
            }
        };

        let mut interpreter = match Interpreter::new().load_neander_file(&bin) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("Error converting bin to Neander format: {}", e);
                continue;
            }
        };

        interpreter.run();
        std::println!("{}", interpreter.acc);
    }
}

fn compile_and_export(data: &str, output_path: Option<String>) -> Result<(), Box<dyn Error>> {
    let l = LexerCalc::new().start_stream(data);
    let mut parser = CalcParser::new(l)?;
    let mut codegen = CodegenCalc::new();
    let ast = parser.parse_expr()?;
    codegen.generate_code(&ast);

    if let Some(path) = output_path {
        let mut p = PathBuf::from(path);

        if p.is_dir() {
            p.push("calc_out.asm");
        } else if p.extension().is_none() {
            p.set_extension(".asm");
        }

        fs::write(&p, codegen.code)?;
        eprintln!("File saved at: {}", p.display());
    } else {
        let mut handle = io::stdout().lock();
        handle.write_all(codegen.code.as_bytes())?;
        handle.flush()?;
    }

    Ok(())
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    if let Some(path) = cli.path {
        let data = fs::read_to_string(path)?;
        compile_and_export(&data, cli.output)?;
    } else {
        process_expression();
    }

    Ok(())
}
