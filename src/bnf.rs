use bnf::Grammar;
use std::error::Error;
use std::fs;

pub fn main() -> Result<(), Box<dyn Error>> {
    let input = fs::read_to_string("utils/grammar.txt")?;
    let grammar: Grammar = input.parse().unwrap();
    for _ in 0..5 {
        let setence = grammar.generate()?;
        println!("{}", setence);
    }

    Ok(())
}
